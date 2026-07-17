//! Throughput EMA and per-worker rate state.

use crate::core::ports::ProgressSnapshot;
use std::sync::Mutex;
use std::time::Instant;

pub struct RateSample {
    pub at: Instant,
    pub logical: u64,
    pub written: u64,
    pub in_bps: u64,
    pub out_bps: u64,
}

impl RateSample {
    pub fn new(now: Instant) -> Self {
        Self {
            at: now,
            logical: 0,
            written: 0,
            in_bps: 0,
            out_bps: 0,
        }
    }
}

#[derive(Clone)]
pub struct WorkerView {
    pub phase: &'static str,
    pub current: String,
    pub logical: u64,
    pub rate_at: Instant,
    pub rate_logical: u64,
    pub bps: u64,
    pub active: bool,
}

impl Default for WorkerView {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            phase: "idle",
            current: String::new(),
            logical: 0,
            rate_at: now,
            rate_logical: 0,
            bps: 0,
            active: false,
        }
    }
}

/// Update aggregate in/out B/s from snapshot; may set `work_started` on first real work.
pub fn update_rates(
    rate: &Mutex<RateSample>,
    work_started: &Mutex<Option<Instant>>,
    snap: &ProgressSnapshot,
) -> (u64, u64) {
    let now = Instant::now();
    let mut rate = rate.lock().unwrap_or_else(|e| e.into_inner());

    if matches!(
        snap.phase,
        "hash" | "compress" | "transfer" | "write-map" | "done" | "start" | "verify" | "extract"
    ) && (snap.logical_done > 0 || snap.written_done > 0 || snap.phase == "hash")
    {
        if let Ok(mut ws) = work_started.lock() {
            if ws.is_none()
                && matches!(snap.phase, "hash" | "compress" | "verify" | "extract" | "list")
            {
                *ws = Some(now);
                rate.at = now;
                rate.logical = snap.logical_done;
                rate.written = snap.written_done;
            }
        }
    }

    let dt = now.duration_since(rate.at).as_secs_f64();
    if dt >= 0.25 {
        let d_in = snap.logical_done.saturating_sub(rate.logical);
        let d_out = snap.written_done.saturating_sub(rate.written);
        let inst_in = (d_in as f64 / dt) as u64;
        let inst_out = (d_out as f64 / dt) as u64;
        rate.in_bps = if rate.in_bps == 0 {
            inst_in
        } else {
            (rate.in_bps * 3 + inst_in) / 4
        };
        rate.out_bps = if rate.out_bps == 0 {
            inst_out
        } else {
            (rate.out_bps * 3 + inst_out) / 4
        };
        rate.at = now;
        rate.logical = snap.logical_done;
        rate.written = snap.written_done;
    }

    let in_bps = if rate.in_bps > 0 {
        rate.in_bps
    } else if let Ok(ws) = work_started.lock() {
        if let Some(t0) = *ws {
            let e = now.duration_since(t0).as_secs_f64().max(1e-6);
            (snap.logical_done as f64 / e) as u64
        } else {
            0
        }
    } else {
        0
    };
    let out_bps = if rate.out_bps > 0 {
        rate.out_bps
    } else if let Ok(ws) = work_started.lock() {
        if let Some(t0) = *ws {
            let e = now.duration_since(t0).as_secs_f64().max(1e-6);
            (snap.written_done as f64 / e) as u64
        } else {
            0
        }
    } else {
        0
    };
    (in_bps, out_bps)
}

pub fn update_worker(w: &mut WorkerView, phase: &'static str, current: &str, logical_delta: u64) {
    let now = Instant::now();
    w.phase = phase;
    w.current = current.to_string();
    w.active = phase != "idle";
    w.logical = w.logical.saturating_add(logical_delta);

    let dt = now.duration_since(w.rate_at).as_secs_f64();
    if dt >= 0.2 {
        let d = w.logical.saturating_sub(w.rate_logical);
        let inst = (d as f64 / dt) as u64;
        w.bps = if w.bps == 0 {
            inst
        } else {
            (w.bps * 3 + inst) / 4
        };
        w.rate_at = now;
        w.rate_logical = w.logical;
    }
}
