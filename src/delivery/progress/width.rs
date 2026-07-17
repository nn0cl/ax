//! Terminal display width helpers (pure; no I/O except optional env in probe).
//!
//! Geometry: **ADR 0024** — Linux console / VT baseline **80 columns**,
//! optional **wide** mode at ≥ 120 columns.

/// Minimum / design terminal width: classic Linux console / VT text mode.
pub const MIN_CONSOLE_COLS: usize = 80;

/// Soft reference for small-screen height (Linux console often 80×24).
/// Public for ADR 0024 / docs; not all call sites need height clamping yet.
#[allow(dead_code)]
pub const MIN_CONSOLE_ROWS: usize = 24;

/// Max columns for one progress row under the design baseline (= 80).
pub const ROW_COLS: usize = MIN_CONSOLE_COLS;

/// Unlock wider dashboard rows (archive + source on `cmp`) at this width.
pub const WIDE_CONSOLE_COLS: usize = 120;

/// Cap for wide mode so ultra-wide terminals do not paint endless lines.
pub const MAX_ROW_COLS: usize = 200;

/// Soft default for path segments when a fixed budget is requested.
/// Worker (`cmp`) rows prefer **all residual columns** after metrics so the
/// compression-target file name stays readable (see `truncate_path_keep_name`).
pub const PATH_COLS: usize = 40;

/// Floor for best-effort paint on very narrow terminals (not a design target).
const ABS_MIN_ROW_COLS: usize = 40;

/// Effective row width for painting.
///
/// - Design baseline: **80** ([`MIN_CONSOLE_COLS`]) for TTY &lt; 120.
/// - **Wide** TTY (≥ [`WIDE_CONSOLE_COLS`]): use detected width up to
///   [`MAX_ROW_COLS`] so `cmp` can show archive **and** source file.
/// - Narrower than 80: best-effort only (not certified).
pub fn effective_row_cols(terminal_cols: Option<usize>) -> usize {
    match terminal_cols {
        Some(c) if c >= WIDE_CONSOLE_COLS => c.min(MAX_ROW_COLS),
        Some(c) if c >= ABS_MIN_ROW_COLS => c.min(MIN_CONSOLE_COLS),
        Some(c) if c > 0 => ABS_MIN_ROW_COLS.min(MIN_CONSOLE_COLS),
        _ => MIN_CONSOLE_COLS,
    }
}

/// True when the paint width can host dual paths on `cmp` rows.
#[inline]
pub fn is_wide_layout(row_cols: usize) -> bool {
    row_cols >= WIDE_CONSOLE_COLS
}

/// Path column budget derived from row width (metrics prefix needs room).
/// Soft hint only; `finish_row` uses residual columns after the metrics prefix.
pub fn path_cols_for_row(row_cols: usize) -> usize {
    // ~ half of row after short labels, clamped.
    (row_cols / 2).clamp(16, PATH_COLS)
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

/// Last path component (`/` or `\\` separated).
pub fn file_name(s: &str) -> &str {
    s.rsplit(['/', '\\'])
        .find(|p| !p.is_empty())
        .unwrap_or(s)
}

/// Width of `·` + 4 hex digits in [`display_file_label`].
pub const PATH_TAG_COLS: usize = 5;

/// Stable 4-hex tag from the **full** relative path (FNV-1a).
///
/// Not cryptographic randomness — deterministic so the same path always shows the
/// same tag (logs stay comparable; lists stay sortable by full path elsewhere).
/// Disambiguates same basenames (`a/report.docx` vs `b/report.docx` → different tags).
pub fn path_tag(s: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in s.bytes() {
        // Normalize separators so `a\b` and `a/b` share a tag on mixed inputs.
        let b = if b == b'\\' { b'/' } else { b };
        h ^= u32::from(b);
        h = h.wrapping_mul(16_777_619);
    }
    format!("{:04x}", h & 0xffff)
}

/// Dashboard path label: **file name + short path tag**.
///
/// Example: `part-0008.zip·a3f2` — directories are omitted, but same basenames from
/// different folders do not look identical (tag is derived from the full path).
/// Prefer this over pure random IDs: tags are stable across runs and do not
/// disturb sort order of the underlying `rel_path` lists.
pub fn display_file_label(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    let name = file_name(s);
    let tag = path_tag(s);
    // `·xxxx` is always kept when budget allows so collisions stay visible.
    if max_cols <= PATH_TAG_COLS {
        return take_cols_end(&format!("·{tag}"), max_cols);
    }
    let name_budget = max_cols - PATH_TAG_COLS;
    let name_part = if display_width(name) <= name_budget {
        name.to_string()
    } else {
        let ell = '…';
        let ell_w = char_cols(ell);
        if name_budget <= ell_w {
            take_cols_end(name, name_budget)
        } else {
            format!("{ell}{}", take_cols_end(name, name_budget - ell_w))
        }
    };
    format!("{name_part}·{tag}")
}

/// Fit a path into `max_cols`, **always preferring the file name**.
///
/// Prefer [`display_file_label`] for progress rows. This keeps a path suffix
/// when a caller still wants parent context.
#[allow(dead_code)] // kept for callers that want parent+name suffix
pub fn truncate_path_keep_name(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if display_width(s) <= max_cols {
        return s.to_string();
    }
    let ell = '…';
    let ell_w = char_cols(ell);
    if max_cols <= ell_w {
        return take_cols_end(s, max_cols);
    }
    let budget = max_cols - ell_w;
    let name = file_name(s);
    if display_width(name) > budget {
        return format!("{ell}{}", take_cols_end(name, budget));
    }
    format!("{ell}{}", take_cols_end(s, budget))
}

/// Hard-cap a finished row to [`ROW_COLS`] (80). Public alias for ADR 0024 default.
#[inline]
#[allow(dead_code)] // used by tests; call sites prefer `fit_row_to` with Layout
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

/// Keep the left side of `s` up to `max_cols` display columns (no ellipsis).
/// Used when a metrics prefix itself overflows a narrow terminal — prefer
/// preserving leading numbers over middle-mangling (`…TA` from `ETA`).
pub fn take_cols_left(s: &str, max_cols: usize) -> String {
    take_cols(s, max_cols)
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
    fn effective_caps_and_wide_mode() {
        assert_eq!(effective_row_cols(None), 80);
        assert_eq!(effective_row_cols(Some(80)), 80);
        assert_eq!(effective_row_cols(Some(100)), 80); // below wide threshold → 80 cap
        assert_eq!(effective_row_cols(Some(120)), 120);
        assert_eq!(effective_row_cols(Some(140)), 140);
        assert_eq!(effective_row_cols(Some(200)), 200);
        assert_eq!(effective_row_cols(Some(250)), MAX_ROW_COLS);
        assert_eq!(effective_row_cols(Some(60)), 60);
        assert_eq!(effective_row_cols(Some(20)), ABS_MIN_ROW_COLS);
        assert!(is_wide_layout(120));
        assert!(!is_wide_layout(80));
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

    #[test]
    fn path_keep_name_preserves_basename() {
        let p = "Documents/backup/deep/folder/part-0004.zip";
        let t = truncate_path_keep_name(p, 20);
        assert!(display_width(&t) <= 20);
        assert!(
            t.ends_with("part-0004.zip") || t.contains("part-0004.zip"),
            "basename lost: {t}"
        );
        assert!(!t.starts_with("Documents…"), "got {t}");
    }

    #[test]
    fn display_file_label_basename_plus_stable_tag() {
        let a = display_file_label("Documents/2025_NBOM/VirtualMachine_BOM/part-0008.zip", 40);
        assert!(a.starts_with("part-0008.zip·"), "{a}");
        assert_eq!(
            display_width(&a),
            display_width("part-0008.zip") + PATH_TAG_COLS
        );
        assert_eq!(
            display_file_label("Documents/…/part-0008.zip", 40),
            display_file_label("Documents/…/part-0008.zip", 40)
        );
        // Same basename, different folders → different tags (avoid look-alike accidents).
        let t1 = display_file_label("a/report.docx", 40);
        let t2 = display_file_label("b/report.docx", 40);
        assert_ne!(t1, t2, "same basename must not share label");
        assert!(t1.starts_with("report.docx·"));
        assert!(t2.starts_with("report.docx·"));
        // Stable across calls.
        assert_eq!(path_tag("a/report.docx"), path_tag("a/report.docx"));
        assert_eq!(path_tag(r"a\report.docx"), path_tag("a/report.docx"));
    }

    #[test]
    fn display_file_label_respects_budget() {
        let long = format!("prefix-{}-end.dat", "x".repeat(80));
        let t = display_file_label(&format!("dir/{long}"), 20);
        assert!(display_width(&t) <= 20);
        assert!(t.contains('·'), "{t}");
    }

    #[test]
    fn file_name_extracts_component() {
        assert_eq!(file_name("a/b/c.zip"), "c.zip");
        assert_eq!(file_name("c.zip"), "c.zip");
        assert_eq!(file_name(r"a\b\c.zip"), "c.zip");
    }
}
