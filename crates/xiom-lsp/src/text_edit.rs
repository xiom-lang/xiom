// XIOM Language Server -- Text editing utility for incremental sync
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

pub fn apply_text_edit(text: &str, start_line: usize, start_char: usize, end_line: usize, end_char: usize, new_text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();

    let start_offset = lines.iter()
        .take(start_line)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        .min(text.len())
        + start_char.min(lines.get(start_line).map_or(0, |l| l.len()));

    let end_offset = lines.iter()
        .take(end_line)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        .min(text.len())
        + end_char.min(lines.get(end_line).map_or(0, |l| l.len()));

    let start_offset = start_offset.min(text.len());
    let end_offset = end_offset.max(start_offset).min(text.len());

    let mut result = String::with_capacity(text.len() + new_text.len().saturating_sub(end_offset - start_offset));
    result.push_str(&text[..start_offset]);
    result.push_str(new_text);
    result.push_str(&text[end_offset..]);
    result
}
