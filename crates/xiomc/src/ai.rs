// XIOM — AI-Assisted Compilation Pipeline (Phase 5g MVP)
use std::collections::HashMap;
use std::io::Write;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub enabled: bool,
    pub local_only: bool,
    pub dry_run: bool,
    pub silent: bool,
    pub strict: bool,
    pub model: String,
    pub endpoint: String,
    pub api_key: String,
    pub timeout_secs: u32,
    pub max_tokens: usize,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false, local_only: false, dry_run: false, silent: false, strict: false,
            model: std::env::var("XIOM_AI_MODEL").unwrap_or_else(|_| "codellama".to_string()),
            endpoint: std::env::var("XIOM_AI_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            api_key: std::env::var("XIOM_AI_KEY").unwrap_or_default(),
            timeout_secs: 10, max_tokens: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiHint {
    pub file: String, pub line: u32, pub column: u32,
    pub error_code: String, pub error_type: String,
    pub contract: Option<String>, pub insight: String,
    pub cached: bool, pub timestamp_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")] pub is_root_cause: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")] pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOutput {
    pub schema_version: u32, pub session: String, pub compiler_version: String,
    pub model: String, pub source_hash: String,
    pub total_hints: usize, pub cached_hints: usize, pub api_calls: usize,
    pub hints: Vec<AiHint>,
}

pub struct AiCache { cache_dir: std::path::PathBuf }

impl AiCache {
    pub fn new() -> Self {
        let dir = std::path::PathBuf::from(".xiom_ai_cache");
        let _ = std::fs::create_dir_all(&dir);
        Self { cache_dir: dir }
    }
    fn cache_key(model: &str, error_code: &str, fn_hash: &str, line: u32) -> String {
        let mut h = Sha256::new(); h.update(format!("{model}:{error_code}:{fn_hash}:{line}")); format!("{:x}", h.finalize())
    }
    pub fn get(&self, model: &str, error_code: &str, fn_hash: &str, line: u32) -> Option<AiHint> {
        let key = Self::cache_key(model, error_code, fn_hash, line);
        if let Ok(data) = std::fs::read_to_string(self.cache_dir.join(format!("{key}.json"))) {
            if let Ok(hint) = serde_json::from_str::<AiHint>(&data) { return Some(hint); }
        }
        None
    }
    pub fn put(&self, model: &str, error_code: &str, fn_hash: &str, line: u32, hint: &AiHint) {
        let key = Self::cache_key(model, error_code, fn_hash, line);
        if let Ok(json) = serde_json::to_string(hint) { let _ = std::fs::write(self.cache_dir.join(format!("{key}.json")), json); }
    }
}

pub struct ContextSlice {
    pub function_signature: String, pub function_body: String,
    pub error_code: String, pub error_type: String,
    pub error_line: u32, pub error_column: u32,
    pub contract_clause: Option<String>,
}

pub fn slice_error_context(source: &str, diag: &crate::Diagnostic) -> Option<ContextSlice> {
    let lines: Vec<&str> = source.lines().collect();
    let el = diag.line.saturating_sub(1) as usize;
    let mut fn_sig = String::new(); let mut fn_start = el;
    for i in (0..=el).rev() {
        let line = lines.get(i).unwrap_or(&"");
        if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") { fn_start = i; fn_sig = line.trim().to_string(); break; }
    }
    let mut body = String::new(); let mut bc = 0i32; let mut fo = false;
    for i in fn_start..lines.len() {
        let line = lines.get(i).unwrap_or(&""); if line.contains('{') { fo = true; }
        if fo { body.push_str(line); body.push('\n');
            for ch in line.chars() { if ch == '{' { bc += 1; } if ch == '}' { bc -= 1; } }
            if bc == 0 && fo { break; }
        }
        if body.split_whitespace().count() > 200 { body.push_str("  // ... (truncated)\n"); break; }
    }
    let contract = if diag.message.contains("requires") || diag.message.contains("ensures") {
        let idx = diag.message.find("requires:").or_else(|| diag.message.find("ensures:"));
        idx.map(|i| diag.message[i..].lines().next().unwrap_or("").trim().to_string())
    } else { None };
    let et = match diag.code.chars().next().unwrap_or('?') {
        'X' => "ContractViolation", 'T' => "TypeError", 'P' => "ParseError", 'C' => "CodegenError", _ => "CompileError",
    };
    Some(ContextSlice { function_signature: fn_sig, function_body: body, error_code: diag.code.clone(),
        error_type: et.to_string(), error_line: diag.line, error_column: diag.col, contract_clause: contract })
}

const DEFAULT_PROMPT: &str = r#"[System] You are the internal 'xiomc' compiler diagnostic translator. Give 1-2 sentence insights. CRITICAL: Do not write code.
[State] Error: {error_code} ({error_type}) Line: {line}
[Code]
```xiom
{function_body}
```
[Task] Explain the failure in 15-45 words. Be specific about which variable or expression triggered it."#;

fn build_prompt(ctx: &ContextSlice) -> String {
    DEFAULT_PROMPT
        .replace("{error_code}", &ctx.error_code).replace("{error_type}", &ctx.error_type)
        .replace("{line}", &ctx.error_line.to_string()).replace("{function_body}", &ctx.function_body)
}

pub fn run_ai_pipeline(config: &AiConfig, source: &str, source_path: &str, diagnostics: &[crate::Diagnostic]) -> Result<AiOutput, String> {
    if diagnostics.is_empty() {
        return Ok(AiOutput { schema_version: 1, session: String::new(), compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            model: config.model.clone(), source_hash: hash_source(source), total_hints: 0, cached_hints: 0, api_calls: 0, hints: vec![] });
    }
    if !config.local_only && config.api_key.is_empty() && config.endpoint.contains("11434") {
        return Err("--ai requires XIOM_AI_KEY or a local LLM.\nSet XIOM_AI_KEY=<key> or use --ai-local for offline mode.".to_string());
    }
    let cache = AiCache::new();
    let mut hints = Vec::new(); let mut cached = 0usize; let mut api_calls = 0usize;

    for diag in diagnostics {
        let ctx = match slice_error_context(source, diag) { Some(c) => c, None => continue };
        let fn_hash = hash_str(&ctx.function_body);
        if let Some(mut hint) = cache.get(&config.model, &ctx.error_code, &fn_hash, ctx.error_line) {
            hint.is_root_cause = Some(hints.is_empty()); hints.push(hint); cached += 1; continue;
        }
        let prompt = build_prompt(&ctx);
        let insight = if config.dry_run {
            eprintln!("[AI DRY RUN] Prompt for {}:{}:\n{prompt}", source_path, ctx.error_line);
            "(dry run — no LLM call)".to_string()
        } else {
            match call_ollama(&config.endpoint, &config.model, &prompt, config.timeout_secs) {
                Ok(text) => { api_calls += 1; text }
                Err(e) => { eprintln!("[AI] LLM call failed: {e}"); format!("[fallback] {}", diag.message) }
            }
        };
        let hint = AiHint { file: source_path.to_string(), line: diag.line, column: diag.col,
            error_code: diag.code.clone(), error_type: ctx.error_type.clone(), contract: ctx.contract_clause.clone(),
            insight, cached: false,
            timestamp_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
            is_root_cause: Some(hints.is_empty()),
            confidence: match ctx.error_type.as_str() { "ContractViolation" => Some("HIGH".to_string()), _ => Some("MEDIUM".to_string()) },
        };
        cache.put(&config.model, &ctx.error_code, &fn_hash, ctx.error_line, &hint);
        hints.push(hint);
    }
    hints.sort_by_key(|h| (h.file.clone(), h.line));
    let output = AiOutput { schema_version: 1, session: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
        compiler_version: env!("CARGO_PKG_VERSION").to_string(), model: config.model.clone(),
        source_hash: hash_source(source), total_hints: hints.len(), cached_hints: cached, api_calls, hints };
    let json = serde_json::to_string_pretty(&output).map_err(|e| format!("JSON error: {e}"))?;
    std::fs::write(".xiom_ai.json", &json).map_err(|e| format!("Write error: {e}"))?;
    if !config.silent { eprintln!("xiomc --ai: {} hints written to .xiom_ai.json ({} API, {} cached)", output.total_hints, api_calls, cached); }
    Ok(output)
}

fn call_ollama(endpoint: &str, model: &str, prompt: &str, timeout: u32) -> Result<String, String> {
    let body = serde_json::json!({ "model": model, "prompt": prompt, "stream": false, "options": { "temperature": 0.0, "num_predict": 100 } });
    let resp = ureq::post(&format!("{endpoint}/api/generate")).timeout(std::time::Duration::from_secs(timeout as u64))
        .send_json(&body).map_err(|e| format!("Ollama error: {e}"))?;
    let json: serde_json::Value = resp.into_json().map_err(|e| format!("Parse error: {e}"))?;
    json["response"].as_str().map(|s| s.to_string()).ok_or_else(|| "No response".to_string())
}

fn hash_source(s: &str) -> String { let mut h = Sha256::new(); h.update(s); format!("{:x}", h.finalize())[..8].to_string() }
fn hash_str(s: &str) -> String { let mut h = Sha256::new(); h.update(s); format!("{:x}", h.finalize())[..16].to_string() }
