//! Terminal display width helpers (pure; no I/O except optional env in probe).
//!
//! Geometry: **ADR 0024** — Linux console / VT baseline **80 columns**.

/// Minimum / design terminal width: classic Linux console / VT text mode.
pub const MIN_CONSOLE_COLS: usize = 80;

/// Soft reference for small-screen height (Linux console often 80×24).
/// Public for ADR 0024 / docs; not all call sites need height clamping yet.
#[allow(dead_code)]
pub const MIN_CONSOLE_ROWS: usize = 24;

/// Max columns for one progress row under the design baseline (= 80).
pub const ROW_COLS: usize = MIN_CONSOLE_COLS;

/// Default columns for a path segment on a dashboard line.
pub const PATH_COLS: usize = 20;

/// Floor for best-effort paint on very narrow terminals (not a design target).
const ABS_MIN_ROW_COLS: usize = 40;

/// Effective row width for painting.
///
/// - Design/target: **80** ([`MIN_CONSOLE_COLS`]).
/// - Wider TTY: still **cap at 80** (ADR 0024).
/// - Narrower TTY: `min(detected, 80)` best-effort (not certified).
pub fn effective_row_cols(terminal_cols: Option<usize>) -> usize {
    match terminal_cols {
        Some(c) if c >= ABS_MIN_ROW_COLS => c.min(MIN_CONSOLE_COLS),
        Some(c) if c > 0 => ABS_MIN_ROW_COLS.min(MIN_CONSOLE_COLS),
        _ => MIN_CONSOLE_COLS,
    }
}

/// Path column budget derived from row width (metrics prefix needs room).
pub fn path_cols_for_row(row_cols: usize) -> usize {
    // ~1/4 of row, clamped.
    (row_cols / 4).clamp(12, PATH_COLS)
}

/// Approximate display width: ASCII = 1, most CJK/fullwidth = 2.
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_cols).sum()
}

fn char_cols(c: char) -> usize {
    match c {
        '\0' => 0,
        c if c < '\u{1100}' => 1,
        _ => 2,
    }
}

/// Truncate to `max_cols` display columns, with `…` in the middle.
pub fn truncate_middle(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if display_width(s) <= max_cols {
        return s.to_string();
    }
    if max_cols < 5 {
        return take_cols(s, max_cols);
    }
    let ell = '…';
    let budget = max_cols.saturating_sub(char_cols(ell));
    let head_cols = budget / 2;
    let tail_cols = budget - head_cols;
    format!(
        "{}{}{}",
        take_cols(s, head_cols),
        ell,
        take_cols_end(s, tail_cols)
    )
}

/// Hard-cap a finished row to [`ROW_COLS`] (80).
#[inline]
pub fn fit_row(s: impl Into<String>) -> String {
    fit_row_to(s, ROW_COLS)
}

/// Hard-cap a finished row to `row_cols` display columns.
pub fn fit_row_to(s: impl Into<String>, row_cols: usize) -> String {
    let s = s.into();
    let max = row_cols.max(1);
    if display_width(&s) <= max {
        s
    } else {
        truncate_middle(&s, max)
    }
}

fn take_cols(s: &str, max_cols: usize) -> String {
    let mut out = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = char_cols(c);
        if w + cw > max_cols {
            break;
        }
        out.push(c);
        w += cw;
    }
    out
}

fn take_cols_end(s: &str, max_cols: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut w = 0usize;
    for &c in chars.iter().rev() {
        let cw = char_cols(c);
        if w + cw > max_cols {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.into_iter().rev().collect()
}

/// Best-effort terminal column count: `COLUMNS` env, then Unix `TIOCGWINSZ`.
pub fn probe_terminal_columns() -> Option<usize> {
    if let Ok(s) = std::env::var("COLUMNS") {
        if let Ok(n) = s.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }
    }
    probe_tty_columns_ioctl()
}

#[cfg(unix)]
fn probe_tty_columns_ioctl() -> Option<usize> {
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    let file = OpenOptions::new().read(true).write(true).open("/dev/tty").ok()?;
    let mut ws = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: ioctl with winsize buffer; fd is open /dev/tty.
    let rc = unsafe { libc::ioctl(file.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
    if rc == 0 && ws.ws_col > 0 {
        Some(ws.ws_col as usize)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn probe_tty_columns_ioctl() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_is_linux_80() {
        assert_eq!(MIN_CONSOLE_COLS, 80);
        assert_eq!(ROW_COLS, 80);
        assert_eq!(MIN_CONSOLE_ROWS, 24);
    }

    #[test]
    fn effective_caps_at_80() {
        assert_eq!(effective_row_cols(None), 80);
        assert_eq!(effective_row_cols(Some(120)), 80);
        assert_eq!(effective_row_cols(Some(80)), 80);
        assert_eq!(effective_row_cols(Some(60)), 60);
        assert_eq!(effective_row_cols(Some(20)), ABS_MIN_ROW_COLS);
    }

    #[test]
    fn ascii_path_truncate() {
        let s = truncate_middle("docs/very/long/path/part-0001.tar.zst", 20);
        assert!(display_width(&s) <= 20);
        assert!(s.contains('…'));
    }

    #[test]
    fn cjk_uses_double_width() {
        let path = "詳細設計書/ブロック情報管理/part-0001.zip";
        let t = truncate_middle(path, 16);
        assert!(display_width(&t) <= 16);
        let row = fit_row(format!("total     scan  1 files  elapsed 1s  {path}"));
        assert!(display_width(&row) <= ROW_COLS);
    }

    #[test]
    fn fit_row_to_respects_custom_width() {
        let row = fit_row_to("abcdefghijklmnopqrstuvwxyz0123456789", 10);
        assert!(display_width(&row) <= 10);
    }

    #[test]
    fn fit_row_defaults_to_80() {
        let long = "a".repeat(200);
        assert!(display_width(&fit_row(long)) <= ROW_COLS);
    }
}
