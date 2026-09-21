// XIOM Language Server -- AI insight integration for hover
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Search `.xiom_ai.json` for a hint matching the given file and line (0-indexed).
/// Returns the AI insight text if found, or None.
pub fn get_ai_insight_for_line(uri: &str, line: usize) -> Option<String> {
    let ai_path = std::path::Path::new(".xiom_ai.json");
    if !ai_path.exists() { return None; }

    let data = std::fs::read_to_string(ai_path).ok()?;
    let output: serde_json::Value = serde_json::from_str(&data).ok()?;

    let file_name = uri.rsplit('/').next().unwrap_or(uri);

    let hints = output["hints"].as_array()?;
    for hint in hints {
        let hint_file = hint["file"].as_str().unwrap_or("");
        let hint_line = hint["line"].as_u64().unwrap_or(0) as usize;
        if (hint_file.ends_with(file_name) || hint_file == file_name) && hint_line.saturating_sub(1) == line {
            let insight = hint["insight"].as_str().unwrap_or("");
            if insight.is_empty() || insight.starts_with("[fallback]") { return None; }
            let confidence = hint["confidence"].as_str().unwrap_or("MEDIUM");
            let error_code = hint["error_code"].as_str().unwrap_or("");
            return Some(format!("[{error_code}] ({confidence} confidence)\n{insight}"));
        }
    }
    None
}
