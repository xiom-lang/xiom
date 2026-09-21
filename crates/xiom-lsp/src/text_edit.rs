// XIOM Language Server -- Text editing utility for incremental sync
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Byte offset of an LSP (zero-based line, UTF-16 character) position.
/// Clamps to the document end and always lands on a char boundary, so
/// incremental didChange edits can never panic on multibyte content.
fn position_to_byte(text: &str, line0: usize, utf16: usize) -> usize {
    let mut offset = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        if i == line0 {
            return offset + crate::position::utf16_to_byte(line, utf16);
        }
        offset += line.len() + 1;
        if offset > text.len() {
            return text.len();
        }
    }
    text.len()
}

/// Apply an LSP incremental change. `start_char`/`end_char` are UTF-16 code
/// units (spec), not bytes: the old implementation sliced by raw bytes and
/// panicked ("byte index is not a char boundary") on any multibyte document.
pub fn apply_text_edit(
    text: &str,
    start_line: usize,
    start_char: usize,
    end_line: usize,
    end_char: usize,
    new_text: &str,
) -> String {
    let start_offset = position_to_byte(text, start_line, start_char).min(text.len());
    let end_offset = position_to_byte(text, end_line, end_char)
        .max(start_offset)
        .min(text.len());
    let mut result =
        String::with_capacity(text.len() + new_text.len().saturating_sub(end_offset - start_offset));
    result.push_str(&text[..start_offset]);
    result.push_str(new_text);
    result.push_str(&text[end_offset..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multibyte_edits_stay_on_char_boundaries() {
        let text = "let s = \"\u{03A9}\u{1F98A}\";\nlet t = 1;\n";
        // Replace the whole first line (LSP UTF-16 offsets).
        let line0 = "let s = \"\u{03A9}\u{1F98A}\";";
        let units: usize = line0.chars().map(|c| c.len_utf16()).sum();
        let out = apply_text_edit(text, 0, 0, 0, units, "let s = \"ok\";");
        assert_eq!(out, "let s = \"ok\";\nlet t = 1;\n");
        // Insert in the middle of the multibyte literal without panicking.
        let out2 = apply_text_edit(text, 0, 9, 0, 9, "X");
        assert!(out2.starts_with("let s = \"X"));
    }

    #[test]
    fn past_end_positions_clamp() {
        let text = "fn main() {}\n";
        let out = apply_text_edit(text, 99, 99, 99, 99, "// tail");
        assert_eq!(out, "fn main() {}\n// tail");
    }
}
