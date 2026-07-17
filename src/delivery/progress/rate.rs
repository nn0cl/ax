//! Throughput estimates from progress snapshots (pure-ish state; no I/O).

use crate::core::ports::ProgressSnapshot;
use std::time::Instant;

/// Exponential moving average of aggregate in/out byte rates.
#[derive(Debug, Clone)]
pub struct RateSample {
    at: Instant,
    logical: u64,
    written: u64,
    in_bps: u64,
    out_bps: u64,
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

    /// Update EMA from `snap`. Returns `(in_bps, out_bps)`.
    pub fn observe(
        &mut self,
        now: Instant,
        work_started: &mut Option<Instant>,
        snap: &ProgressSnapshot,
    ) -> (u64, u64) {
        self.maybe_start_clock(now, work_started, snap);

        let dt = now.duration_since(self.at).as_secs_f64();
        if dt >= 0.25 {
            let d_in = snap.logical_done.saturating_sub(self.logical);
            let d_out = snap.written_done.saturating_sub(self.written);
            self.in_bps = ema(self.in_bps, (d_in as f64 / dt) as u64);
            self.out_bps = ema(self.out_bps, (d_out as f64 / dt) as u64);
            self.at = now;
            self.logical = snap.logical_done;
            self.written = snap.written_done;
        }

        let in_bps = self.in_bps_or_avg(now, work_started, snap.logical_done);
        let out_bps = self.out_bps_or_avg(now, work_started, snap.written_done);
        (in_bps, out_bps)
    }

    fn maybe_start_clock(
        &mut self,
        now: Instant,
        work_started: &mut Option<Instant>,
        snap: &ProgressSnapshot,
    ) {
        let active = matches!(
            snap.phase,
            "hash" | "compress" | "transfer" | "write-map" | "done" | "start" | "verify" | "extract"
        ) && (snap.logical_done > 0 || snap.written_done > 0 || snap.phase == "hash");
        if !active {
            return;
        }
        if work_started.is_none()
            && matches!(snap.phase, "hash" | "compress" | "verify" | "extract" | "list")
        {
            *work_started = Some(now);
            self.at = now;
            self.logical = snap.logical_done;
            self.written = snap.written_done;
        }
    }

    fn in_bps_or_avg(
        &self,
        now: Instant,
        work_started: &Option<Instant>,
        logical_done: u64,
    ) -> u64 {
        if self.in_bps > 0 {
            return self.in_bps;
        }
        work_started
            .map(|t0| {
                let e = now.duration_since(t0).as_secs_f64().max(1e-6);
                (logical_done as f64 / e) as u64
            })
            .unwrap_or(0)
    }

    fn out_bps_or_avg(
        &self,
        now: Instant,
        work_started: &Option<Instant>,
        written_done: u64,
    ) -> u64 {
        if self.out_bps > 0 {
            return self.out_bps;
        }
        work_started
            .map(|t0| {
                let e = now.duration_since(t0).as_secs_f64().max(1e-6);
                (written_done as f64 / e) as u64
            })
            .unwrap_or(0)
    }
}

/// One compress-worker row in the dashboard.
#[derive(Debug, Clone)]
pub struct WorkerView {
    pub phase: &'static str,
    pub current: String,
    pub bps: u64,
    pub active: bool,
    logical: u64,
    rate_at: Instant,
    rate_logical: u64,
}

impl Default for WorkerView {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            phase: "idle",
            current: String::new(),
            bps: 0,
            active: false,
            logical: 0,
            rate_at: now,
            rate_logical: 0,
        }
    }
}

impl WorkerView {
    pub fn tick(&mut self, phase: &'static str, current: &str, logical_delta: u64) {
        let now = Instant::now();
        self.phase = phase;
        self.current = current.to_string();
        self.active = phase != "idle";
        self.logical = self.logical.saturating_add(logical_delta);

        let dt = now.duration_since(self.rate_at).as_secs_f64();
        if dt >= 0.2 {
            let d = self.logical.saturating_sub(self.rate_logical);
            self.bps = ema(self.bps, (d as f64 / dt) as u64);
            self.rate_at = now;
            self.rate_logical = self.logical;
        }
    }
}

fn ema(prev: u64, inst: u64) -> u64 {
    if prev == 0 {
        inst
    } else {
        (prev * 3 + inst) / 4
    }
}
