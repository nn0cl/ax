//! Terminal display width helpers (pure; no I/O).
//!
//! Japanese / fullwidth glyphs typically occupy **2 columns**. Truncating by
//! `char` count lets lines wrap and breaks fixed-height cursor math.

/// Max columns for one progress row (avoid terminal wrap).
pub const ROW_COLS: usize = 72;

/// Columns used by a path segment on a dashboard line.
pub const PATH_COLS: usize = 20;

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

/// Hard-cap a finished row to [`ROW_COLS`].
pub fn fit_row(s: impl Into<String>) -> String {
    let s = s.into();
    if display_width(&s) <= ROW_COLS {
        s
    } else {
        truncate_middle(&s, ROW_COLS)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
