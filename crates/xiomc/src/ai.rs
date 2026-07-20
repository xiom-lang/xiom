// XIOM — AI-Assisted Compilation Pipeline (Phase 5g)
// Supports: Ollama, DeepSeek, OpenAI, OpenRouter, Groq, and any OpenAI-compatible endpoint.
// Secure config via .xiom_ai_config.json or environment variables.
// Never modifies source files. Only writes .xiom_ai.json hints.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

// =========================================================================
// Configuration — loaded from .xiom_ai_config.json, then env vars, then defaults
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigFile {
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,  // "ollama", "deepseek", "openai", "openrouter", "groq", "custom"
}

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
    pub provider: String,
    pub timeout_secs: u32,
    pub max_tokens: usize,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false, local_only: false, dry_run: false, silent: false, strict: false,
            model: String::new(), endpoint: String::new(), api_key: String::new(),
            provider: String::new(), timeout_secs: 30, max_tokens: 800,
        }
    }
}

/// Load AI config from .xiom_ai_config.json, then env vars, then defaults.
/// Priority: CLI flags > env vars > config file > built-in defaults.
pub fn load_ai_config(cli_model: Option<String>) -> AiConfig {
    let mut cfg = AiConfig::default();

    // 1. Try .xiom_ai_config.json in current dir, then home dir
    for dir in &[std::env::current_dir().ok(), dirs::home_dir()] {
        if let Some(d) = dir {
            let path = d.join(".xiom_ai_config.json");
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(file_cfg) = serde_json::from_str::<AiConfigFile>(&data) {
                    if let Some(ep) = file_cfg.endpoint { cfg.endpoint = ep; }
                    if let Some(key) = file_cfg.api_key { cfg.api_key = key; }
                    if let Some(m) = file_cfg.model { cfg.model = m; }
                    if let Some(p) = file_cfg.provider { cfg.provider = p; }
                    break; // first found wins
                }
            }
        }
    }

    // 2. Environment variables override config file
    if let Ok(ep) = std::env::var("XIOM_AI_ENDPOINT") { cfg.endpoint = ep; }
    if let Ok(key) = std::env::var("XIOM_AI_KEY") { cfg.api_key = key; }
    if let Ok(m) = std::env::var("XIOM_AI_MODEL") { cfg.model = m; }
    if let Ok(p) = std::env::var("XIOM_AI_PROVIDER") { cfg.provider = p; }

    // 3. CLI model flag overrides all
    if let Some(m) = cli_model { cfg.model = m; }

    // 4. Auto-detect provider from endpoint if not set
    if cfg.provider.is_empty() {
        cfg.provider = detect_provider(&cfg.endpoint);
    }

    // 5. Apply provider defaults if endpoint/key/model still empty
    if cfg.endpoint.is_empty() {
        cfg.endpoint = default_endpoint(&cfg.provider);
    }
    if cfg.model.is_empty() {
        cfg.model = default_model(&cfg.provider);
    }

    cfg
}

fn detect_provider(endpoint: &str) -> String {
    let ep = endpoint.to_lowercase();
    if ep.contains("11434") || ep.contains("ollama") { return "ollama".into(); }
    if ep.contains("deepseek") { return "deepseek".into(); }
    if ep.contains("openai") { return "openai".into(); }
    if ep.contains("openrouter") { return "openrouter".into(); }
    if ep.contains("groq") { return "groq".into(); }
    if ep.contains("api") || ep.contains("v1") { return "openai-compatible".into(); }
    "ollama".into()
}

fn default_endpoint(provider: &str) -> String {
    match provider {
        "ollama" => "http://localhost:11434".into(),
        "deepseek" => "https://api.deepseek.com".into(),
        "openai" => "https://api.openai.com/v1".into(),
        "openrouter" => "https://openrouter.ai/api/v1".into(),
        "groq" => "https://api.groq.com/openai/v1".into(),
        _ => "http://localhost:11434".into(),
    }
}

fn default_model(provider: &str) -> String {
    match provider {
        "ollama" => "codellama".into(),
        "deepseek" => "deepseek-chat".into(),
        "openai" => "gpt-4o-mini".into(),
        "openrouter" => "anthropic/claude-3.5-sonnet".into(),
        "groq" => "llama-3.1-8b-instant".into(),
        _ => "codellama".into(),
    }
}

// =========================================================================
// AI Hint Output
// =========================================================================

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
    pub provider: String, pub model: String, pub source_hash: String,
    pub total_hints: usize, pub cached_hints: usize, pub api_calls: usize,
    pub hints: Vec<AiHint>,
}

// =========================================================================
// Hash Cache
// =========================================================================

pub struct AiCache { cache_dir: std::path::PathBuf }

impl AiCache {
    pub fn new(model: &str) -> Self {
        let dir = std::path::PathBuf::from(".xiom_ai_cache").join(model);
        let _ = std::fs::create_dir_all(&dir);
        Self { cache_dir: dir }
    }
    fn key(model: &str, error_code: &str, fn_hash: &str, line: u32) -> String {
        let mut h = Sha256::new();
        h.update(format!("{model}:{error_code}:{fn_hash}:{line}"));
        format!("{:x}", h.finalize())
    }
    pub fn get(&self, model: &str, error_code: &str, fn_hash: &str, line: u32) -> Option<AiHint> {
        let path = self.cache_dir.join(Self::key(model, error_code, fn_hash, line) + ".json");
        std::fs::read_to_string(&path).ok()
            .and_then(|d| serde_json::from_str::<AiHint>(&d).ok())
    }
    pub fn put(&self, model: &str, error_code: &str, fn_hash: &str, line: u32, hint: &AiHint) {
        let path = self.cache_dir.join(Self::key(model, error_code, fn_hash, line) + ".json");
        if let Ok(json) = serde_json::to_string(hint) { let _ = std::fs::write(path, json); }
    }
}

// =========================================================================
// Context Slicer
// =========================================================================

pub struct ContextSlice {
    pub function_body: String,
    pub error_code: String, pub error_type: String,
    pub error_line: u32,
    pub contract_clause: Option<String>,
    /// Z3 counter-example values: e.g. {"x": "-1", "b": "0"}
    pub counterexample: Option<Vec<(String, String)>>,
}

pub fn slice_error_context(source: &str, diag: &crate::Diagnostic) -> Option<ContextSlice> {
    let lines: Vec<&str> = source.lines().collect();
    let el = diag.line.saturating_sub(1) as usize;
    let mut fn_start = el;
    for i in (0..=el.min(lines.len().saturating_sub(1))).rev() {
        let line = lines.get(i).unwrap_or(&"");
        if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") { fn_start = i; break; }
    }
    let mut body = String::new(); let mut bc = 0i32; let mut fo = false;
    let end = lines.len().min(fn_start + 200);
    for i in fn_start..end {
        let line = lines.get(i).unwrap_or(&""); if line.contains('{') { fo = true; }
        if fo { body.push_str(line); body.push('\n');
            for ch in line.chars() { if ch == '{' { bc += 1; } if ch == '}' { bc -= 1; } }
            if bc == 0 && fo { break; }
        }
        if body.len() > 3000 { body.push_str("  // ... (truncated)\n"); break; }
    }
    let et = match diag.code.chars().next().unwrap_or('?') {
        'X' => "ContractViolation", 'T' => "TypeError", 'P' => "ParseError", 'C' => "CodegenError", _ => "CompileError",
    };
    // Extract contract clauses from function signature
    let mut contract = None;
    for i in fn_start..el.min(lines.len()) {
        let line = lines.get(i).unwrap_or(&"");
        if line.contains("requires:") || line.contains("ensures:") {
            contract = Some(line.trim().to_string());
        }
    }
    // Also check diagnostic message for contract mentions
    if contract.is_none() && diag.message.contains("contract") {
        contract = Some(diag.message.clone());
    }
    Some(ContextSlice { function_body: body, error_code: diag.code.clone(),
        error_type: et.to_string(), error_line: diag.line, contract_clause: contract,
        counterexample: None })
}

/// Parse z3 model text for counterexample values.
/// Input: "sat\n(model\n  (define-fun x () Int 5)\n  (define-fun |result| () Int (- 5))\n)"
/// Output: [("x", "5"), ("result", "-5")]
pub fn parse_z3_model(model_text: &str) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for line in model_text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("(define-fun ") {
            // Extract name (first token before space or '(')
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(rest.len());
            let name = rest[..name_end].trim_matches('|').to_string();
            // Skip past the empty arg list "()"
            if let Some(after_args) = rest[name_end..].find(')') {
                let after = &rest[name_end + after_args..].trim();
                // Extract value before closing paren
                if let Some(val_end) = after.find(')') {
                    let value = after[..val_end].trim().to_string();
                    if !value.is_empty() && !name.is_empty() {
                        values.push((name, value));
                    }
                }
            }
        }
    }
    values
}

/// Inject counterexample into an AI prompt for context-aware suggestions.
pub fn inject_counterexample(ctx: &mut ContextSlice, z3_output: &str) {
    if z3_output.contains("sat") && z3_output.contains("(define-fun") {
        let ce = parse_z3_model(z3_output);
        if !ce.is_empty() {
            ctx.counterexample = Some(ce);
        }
    }
}

// =========================================================================
// Prompt Templates — provider-optimized
// =========================================================================

/// Load system prompt from stdlib/xiom/ai_prompt.txt, fall back to hardcoded.
fn load_system_prompt() -> String {
    let paths = [
        "stdlib/xiom/ai_prompt.txt",
        "lib/xiom/ai_prompt.txt",
    ];
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if !content.trim().is_empty() {
                return content;
            }
        }
    }
    // Hardcoded fallback (AI-04: production-grade default)
    format!(
        "You are an expert XIOM compiler diagnostic assistant. Your job is to analyze compilation errors and provide SPECIFIC, ACTIONABLE fix suggestions.\n\n\
         RULES:\n\
         - Always suggest the exact fix (e.g., 'change return type from Str to Int' or 'add requires: x != 0')\n\
         - Reference the specific variable or expression that triggered the error\n\
         - If a contract is involved, explain which boundary condition fails\n\
         - Keep responses under 60 words\n\
         - NEVER write full code — suggest the fix in plain English\n\
         - Confidence: HIGH for type/contract errors, MEDIUM for codegen/parse errors\n\n\
         Error categories:\n\
         - T (Type): Type mismatch — check expression type vs declared type\n\
         - C (Codegen): Compiler cannot lower this construct — unsupported pattern\n\
         - P (Parse): Invalid syntax — missing semicolons, braces, or keywords\n\
         - X (Contract): Contract violation — requires/ensures clause not satisfied\n\
         - E (Borrow): Ownership error — use of moved value\n\
         - L (Lexer): Invalid token or character"
    )
}

fn build_chat_prompt(ctx: &ContextSlice) -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"role": "system", "content": load_system_prompt()}),
        serde_json::json!({"role": "user", "content": format!(
            "XIOM Error [{code}] {etype} at line {line}\n\n\
             Code context:\n```xiom\n{body}\n```\n\n\
             {contract_hint}\
             {counterexample_hint}\
             Task: What is the EXACT fix needed? Be specific.",
            code = ctx.error_code, etype = ctx.error_type, line = ctx.error_line,
            body = ctx.function_body,
            contract_hint = ctx.contract_clause.as_ref().map(|c| format!("Failed contract: {c}\n\n")).unwrap_or_default(),
            counterexample_hint = ctx.counterexample.as_ref().map(|ce| {
                let vals: Vec<String> = ce.iter().map(|(k, v)| format!("  {} = {}", k, v)).collect();
                format!("Z3 Counterexample (concrete violation):\n{}\n\n", vals.join("\n"))
            }).unwrap_or_default()
        )}),
    ]
}

// =========================================================================
// LLM Backend — unified OpenAI-compatible chat API
// =========================================================================

fn call_llm_chat(endpoint: &str, api_key: &str, model: &str, messages: &[serde_json::Value], timeout_secs: u32) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.0,
        "max_tokens": 150,
    });

    let mut req = ureq::post(&format!("{endpoint}/chat/completions"))
        .timeout(std::time::Duration::from_secs(timeout_secs as u64));

    if !api_key.is_empty() {
        req = req.set("Authorization", &format!("Bearer {api_key}"));
    }

    let resp = req.send_json(&body).map_err(|e| format!("API error: {e}"))?;
    let json: serde_json::Value = resp.into_json().map_err(|e| format!("Parse error: {e}"))?;

    json["choices"][0]["message"]["content"].as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| format!("Unexpected API response: {}", serde_json::to_string_pretty(&json).unwrap_or_default()))
}

fn call_ollama(endpoint: &str, model: &str, prompt: &str, timeout_secs: u32) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model, "prompt": prompt, "stream": false,
        "options": { "temperature": 0.0, "num_predict": 150 }
    });
    let resp = ureq::post(&format!("{endpoint}/api/generate"))
        .timeout(std::time::Duration::from_secs(timeout_secs as u64))
        .send_json(&body).map_err(|e| format!("Ollama error: {e}"))?;
    let json: serde_json::Value = resp.into_json().map_err(|e| format!("Parse error: {e}"))?;
    json["response"].as_str().map(|s| s.trim().to_string())
        .ok_or_else(|| "No response from Ollama".to_string())
}

// =========================================================================
// Main AI Pipeline
// =========================================================================

pub fn run_ai_pipeline(config: &AiConfig, source: &str, source_path: &str, diagnostics: &[crate::Diagnostic]) -> Result<AiOutput, String> {
    if diagnostics.is_empty() {
        return Ok(AiOutput { schema_version: 1, session: timestamp(), compiler_version: env!("CARGO_PKG_VERSION").into(),
            provider: config.provider.clone(), model: config.model.clone(), source_hash: hash_source(source),
            total_hints: 0, cached_hints: 0, api_calls: 0, hints: vec![] });
    }

    let cache = AiCache::new(&config.model);
    let mut hints = Vec::new(); let mut cached = 0usize; let mut api_calls = 0usize;

    for diag in diagnostics {
        let ctx = match slice_error_context(source, diag) { Some(c) => c, None => continue };
        let fn_hash = hash_str(&ctx.function_body);

        // Check cache
        if let Some(mut hint) = cache.get(&config.model, &ctx.error_code, &fn_hash, ctx.error_line) {
            hint.is_root_cause = Some(hints.is_empty()); hints.push(hint); cached += 1; continue;
        }

        // Call LLM
        let insight = if config.dry_run {
            let prompt = format!("[{}.{}] {}", ctx.error_code, ctx.error_type, ctx.function_body.lines().next().unwrap_or(""));
            eprintln!("[AI DRY RUN] {}:{}:{} → {}", source_path, ctx.error_line, ctx.error_code, prompt);
            "(dry run — no LLM call)".to_string()
        } else {
            let result = if config.provider == "ollama" {
                let prompt = format!("XIOM compiler error [{}] {} at line {}.\nCode:\n```xiom\n{}\n```\nExplain in 1-2 sentences.",
                    ctx.error_code, ctx.error_type, ctx.error_line, ctx.function_body);
                call_ollama(&config.endpoint, &config.model, &prompt, config.timeout_secs)
            } else {
                let messages = build_chat_prompt(&ctx);
                call_llm_chat(&config.endpoint, &config.api_key, &config.model, &messages, config.timeout_secs)
            };
            match result {
                Ok(text) => { api_calls += 1; text }
                Err(e) => { eprintln!("[AI] LLM call failed: {e}"); format!("[fallback] {}", diag.message) }
            }
        };

        let hint = AiHint {
            file: source_path.to_string(), line: diag.line, column: diag.col,
            error_code: diag.code.clone(), error_type: ctx.error_type.clone(),
            contract: ctx.contract_clause.clone(),
            insight, cached: false,
            timestamp_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
            is_root_cause: Some(hints.is_empty()),
            confidence: match ctx.error_type.as_str() { "ContractViolation" | "DivisionByZero" => Some("HIGH".into()), _ => Some("MEDIUM".into()) },
        };
        cache.put(&config.model, &ctx.error_code, &fn_hash, ctx.error_line, &hint);
        hints.push(hint);
    }

    hints.sort_by_key(|h| (h.file.clone(), h.line));
    let output = AiOutput {
        schema_version: 1, session: timestamp(), compiler_version: env!("CARGO_PKG_VERSION").into(),
        provider: config.provider.clone(), model: config.model.clone(),
        source_hash: hash_source(source), total_hints: hints.len(), cached_hints: cached, api_calls, hints,
    };

    let json = serde_json::to_string_pretty(&output).map_err(|e| format!("JSON error: {e}"))?;
    std::fs::write(".xiom_ai.json", &json).map_err(|e| format!("Write error: {e}"))?;

    if !config.silent {
        eprintln!("xiomc --ai: {} hints → .xiom_ai.json ({} API, {} cached, provider: {})",
            output.total_hints, api_calls, cached, config.provider);
    }
    if config.strict && !output.hints.is_empty() {
        return Err(format!("--ai-strict: {} error(s) present. Fix before binary output.", output.total_hints));
    }
    Ok(output)
}

fn timestamp() -> String { format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()) }
fn hash_source(s: &str) -> String { let mut h = Sha256::new(); h.update(s); format!("{:x}", h.finalize())[..8].to_string() }
fn hash_str(s: &str) -> String { let mut h = Sha256::new(); h.update(s); format!("{:x}", h.finalize())[..16].to_string() }

// =========================================================================
// Public API — called from CLI
// =========================================================================

/// Print AI mode help text for --help output
pub fn ai_help_text() -> &'static str {
    r#"
AI-ASSISTED COMPILATION (--ai):
  XIOM can call an LLM to explain compilation errors with actionable hints.
  Results are written to .xiom_ai.json — NEVER modifies source files.

  Quick Start:
    1. Install Ollama:   winget install Ollama.Ollama
    2. Pull a model:      ollama pull codellama
    3. Compile with AI:   xiomc --ai source.xi

  Using DeepSeek:
    set XIOM_AI_KEY=sk-your-deepseek-key
    set XIOM_AI_ENDPOINT=https://api.deepseek.com
    set XIOM_AI_MODEL=deepseek-chat
    xiomc --ai source.xi

  Using OpenAI:
    set XIOM_AI_KEY=sk-your-openai-key
    set XIOM_AI_ENDPOINT=https://api.openai.com/v1
    set XIOM_AI_MODEL=gpt-4o-mini
    xiomc --ai source.xi

  Config File (secure, recommended):
    Create .xiom_ai_config.json in your project or home directory:
    {
      "provider": "deepseek",
      "endpoint": "https://api.deepseek.com",
      "api_key": "sk-your-key-here",
      "model": "deepseek-chat"
    }

  Flags:
    --ai                Enable AI diagnostics (requires Ollama or API key)
    --ai-local          Local-only: never sends code to cloud (Ollama required)
    --ai-dry-run        Print the prompt without calling LLM
    --ai-silent         Suppress stdout, write only .xiom_ai.json
    --ai-strict         Refuse binary output on contract violations
    --ai-model=<name>   Override model (e.g., deepseek-chat, gpt-4o-mini)
    --ai-timeout=<sec>  LLM timeout in seconds (default: 30)

  Supported Providers:
    ollama     (local, free)       codellama, llama3, mistral, phi3
    deepseek   (cloud, cheap)      deepseek-chat, deepseek-coder
    openai     (cloud)             gpt-4o-mini, gpt-4o
    openrouter (cloud, multi)      anthropic/claude-3.5-sonnet, google/gemini-flash
    groq       (cloud, fast)       llama-3.1-8b-instant, mixtral-8x7b

  Environment Variables:
    XIOM_AI_KEY         API key (not needed for Ollama)
    XIOM_AI_ENDPOINT    API endpoint URL
    XIOM_AI_MODEL       Model name (provider-dependent)
    XIOM_AI_PROVIDER    Force provider: ollama, deepseek, openai, openrouter, groq
"#
}
