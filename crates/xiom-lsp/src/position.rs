// XIOM Language Server -- LSP position math
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! LSP positions are zero-based line + UTF-16 code units. XIOM spans carry
//! one-based line + Unicode-scalar columns, and the server's internal string
//! helpers index BYTES. Mixing the three caused multibyte panics
//! (`line_str[..character]` on a non-char boundary) and wrong hover/
//! completion/reference positions. This module is the single conversion
//! point; every conversion clamps, so no input can panic the server.

/// Convert an LSP UTF-16 character offset within `line` to a BYTE offset.
/// Always lands on a char boundary; offsets past the line end clamp to its
/// byte length.
pub fn utf16_to_byte(line: &str, utf16: usize) -> usize {
    let mut units = 0usize;
    for (byte_idx, c) in line.char_indices() {
        if units >= utf16 {
            return byte_idx;
        }
        units += c.len_utf16();
    }
    line.len()
}

/// Convert a BYTE offset within `line` to an LSP UTF-16 character offset.
pub fn byte_to_utf16(line: &str, byte: usize) -> usize {
    line[..byte.min(line.len())]
        .chars()
        .map(|c| c.len_utf16())
        .sum()
}

/// Convert an LSP UTF-16 character offset within `line` to a CHAR index.
/// Clamps past the end (legacy callers index a `Vec<char>`).
pub fn utf16_to_char(line: &str, utf16: usize) -> usize {
    let mut units = 0usize;
    let mut chars = 0usize;
    for c in line.chars() {
        if units >= utf16 {
            break;
        }
        units += c.len_utf16();
        chars += 1;
    }
    chars
}

/// UTF-16 code-unit length of the line at zero-based `line0`.
pub fn line_utf16_len(text: &str, line0: u32) -> u32 {
    text.lines()
        .nth(line0 as usize)
        .map(|l| l.chars().map(|c| c.len_utf16() as u32).sum())
        .unwrap_or(0)
}

/// LSP start position for an XIOM span (one-based line/Unicode-scalar col).
/// The returned `(line, character)` are zero-based line and UTF-16 units,
/// matching `diagnostics::make_range`.
pub fn span_start_lsp(text: &str, span: &xiom_ast::Span) -> (u32, u32) {
    let line0 = span.line.saturating_sub(1);
    let line = text.lines().nth(line0 as usize).unwrap_or("");
    let scalar_col = span.col.saturating_sub(1) as usize;
    let utf16: usize = line
        .chars()
        .take(scalar_col)
        .map(|c| c.len_utf16())
        .sum();
    (line0, utf16 as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_ast::Span;

    #[test]
    fn utf16_conversions_clamp_and_stay_on_boundaries() {
        let line = "let x = \"e\u{03A9}\u{1F98A}\";";
        // Every UTF-16 offset maps to a valid boundary and back monotonically.
        let mut last = 0usize;
        for units in 0..40 {
            let byte = utf16_to_byte(line, units);
            assert!(line.is_char_boundary(byte), "offset {units} -> {byte}");
            assert!(byte >= last, "must be monotonic");
            last = byte;
        }
        assert_eq!(utf16_to_byte(line, 0), 0);
        assert_eq!(utf16_to_byte(line, 10_000), line.len());
        // The two astral chars are surrogate pairs: 2 UTF-16 units each.
        let before_quote = line.find('"').unwrap() + 1;
        let assert_units = line[..before_quote].chars().map(|c| c.len_utf16()).sum::<usize>();
        assert_eq!(utf16_to_byte(line, assert_units), before_quote);
        assert_eq!(byte_to_utf16(line, before_quote), assert_units);
        assert_eq!(byte_to_utf16(line, line.len() + 5), line.chars().map(|c| c.len_utf16()).sum::<usize>());
    }

    #[test]
    fn span_start_uses_utf16_units() {
        let line = "fn main() { let e = \"\u{03A9}\u{1F98A}\"; }";
        let text = format!("{line}\n");
        // One past the last scalar column: UTF-16 length of the whole line.
        let scalar_col = line.chars().count() + 1;
        let span = Span { line: 1, col: scalar_col as u32, byte_start: 0, byte_end: 0 };
        let (l, ch) = span_start_lsp(&text, &span);
        assert_eq!(l, 0);
        assert_eq!(ch as usize, line.chars().map(|c| c.len_utf16()).sum::<usize>());
        // The astral char contributes 2 UTF-16 units: the position right
        // after it must equal the prefix UTF-16 length up to and including it.
        let scalars_before_fox = line.find('\u{1F98A}').map(|b| line[..b].chars().count()).unwrap();
        let prefix_units: usize = line.chars().take(scalars_before_fox + 1).map(|c| c.len_utf16()).sum();
        let span = Span { line: 1, col: (scalars_before_fox + 2) as u32, byte_start: 0, byte_end: 0 };
        let (_, ch2) = span_start_lsp(&text, &span);
        assert_eq!(ch2 as usize, prefix_units);
        // Past-end lines/columns clamp instead of panicking.
        let span = Span { line: 99, col: 99, byte_start: 0, byte_end: 0 };
        assert_eq!(span_start_lsp(&text, &span), (98, 0));
    }
}
