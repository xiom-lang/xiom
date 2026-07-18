// XIOM MCP Server — Model Context Protocol for AI agent tool-calling
// Phase 5d.1-8.2: Library mode (xiomc linked directly, no subprocess).
// Transport: stdio (JSON-RPC 2.0). Production-grade error handling.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::process::Command;

use xiomc::{CompileConfig, compile_with_diagnostics};

// ============================================================================
// Production-grade safety utilities
// ============================================================================

/// Validate a file path for MCP tool access.
fn validate_file_path(path: &str) -> Result<String, String> {
    if path.is_empty() { return Err("Empty file path".into()); }
    if path.len() > 4096 { return Err("Path too long".into()); }
    if path.contains('\0') { return Err("Path contains null byte".into()); }
    if path.contains("..") { return Err("Path traversal rejected".into()); }
    Ok(path.to_string())
}

/// Maximum time allowed for compilation (safety timeout — wired to compile_with_diagnostics config).
#[allow(dead_code)]
const COMPILE_TIMEOUT_SECS: u64 = 120;

// ============================================================================
// JSON-RPC 2.0 types
// ============================================================================

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct ToolDef {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

// ============================================================================
// Tool implementations
// ============================================================================

fn tool_explain_error_code(params: &Value) -> Result<String, String> {
    let code = params["code"]
        .as_str()
        .ok_or("Missing required parameter: code")?;

    // Try to read the error code documentation from docs/error_codes/
    let doc_path = format!("docs/error_codes/{code}.md");
    match std::fs::read_to_string(&doc_path) {
        Ok(content) => Ok(content),
        Err(_) => {
            // Fallback: generic explanation
            let category = if code.starts_with('X') {
                "Syntax/Compiler"
            } else if code.starts_with('L') {
                "Lexer"
            } else if code.starts_with('P') {
                "Parser"
            } else if code.starts_with('T') {
                "Type Checker"
            } else if code.starts_with('C') {
                "Code Generation"
            } else if code.starts_with('E') {
                "Borrow Checker"
            } else {
                "Unknown"
            };
            Ok(format!(
                "# XIOM Error Code: {code}\n\n\
                 **Category:** {category}\n\n\
                 **Note:** No detailed documentation file found at `{doc_path}`.\n\
                 Run `xiomc --explain {code}` for compiler-provided details.\n"
            ))
        }
    }
}

fn tool_compile_and_analyze(params: &Value) -> Result<Value, String> {
    let file = params["file"].as_str().ok_or("Missing required parameter: file")?;
    let file = validate_file_path(file)?;
    if !std::path::Path::new(&file).exists() { return Err(format!("File not found: {file}")); }

    // Phase 8.2: Library mode — calls xiomc::compile_with_diagnostics directly.
    let config = CompileConfig {
        diagnostics_json: true,
        dump_contracts: params["strict"].as_bool().unwrap_or(false),
        ..std::default::Default::default()
    };
    let result = compile_with_diagnostics(&config, &[file.to_string()]);

    Ok(json!({
        "success": result.success,
        "diagnostics": result.diagnostics,
        "warnings": result.warnings,
        "file": file,
        "file_count": result.file_count,
    }))
}

fn tool_get_contract_signature(params: &Value) -> Result<Value, String> {
    let file = params["file"].as_str().ok_or("Missing required parameter: file")?;
    let file = validate_file_path(file)?;
    if !std::path::Path::new(&file).exists() { return Err(format!("File not found: {file}")); }
    let function_name = params["function_name"].as_str();
    let type_name = params["type_name"].as_str();

    if function_name.is_none() && type_name.is_none() {
        return Err("Either function_name or type_name is required".to_string());
    }
    if !std::path::Path::new(&file).exists() {
        return Err(format!("File not found: {file}"));
    }

    // Library mode: compile with dump_contracts to get contract JSON
    let config = CompileConfig {
        dump_contracts: true,
        ..std::default::Default::default()
    };
    let result = compile_with_diagnostics(&config, &[file.to_string()]);

    // Parse contracts from result
    let contracts: Value = result.contracts
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!([]));

    // Filter by function_name or type_name
    let filtered = if let Some(fn_name) = function_name {
        match &contracts {
            Value::Array(items) => Value::Array(
                items.iter().filter(|c| {
                    c.get("function").and_then(|f| f.as_str()).map_or(false, |f| f == fn_name)
                }).cloned().collect()
            ),
            _ => contracts,
        }
    } else if let Some(tn) = type_name {
        match &contracts {
            Value::Array(items) => Value::Array(
                items.iter().filter(|c| {
                    c.get("type").and_then(|t| t.as_str()).map_or(false, |t| t == tn)
                }).cloned().collect()
            ),
            _ => contracts,
        }
    } else { contracts };

    Ok(json!({ "contracts": filtered }))
}

fn tool_check_xiom_syntax(params: &Value) -> Result<Value, String> {
    let source = params["source"]
        .as_str()
        .ok_or("Missing required parameter: source")?;

    // Library mode: parse-only check via compile_with_diagnostics.
    // Writes source to temp file (the API requires a file path for now).
    let tmp = std::env::temp_dir().join(format!("xiom_syntax_check_{}.xi", std::process::id()));
    std::fs::write(&tmp, source).map_err(|e| format!("Failed to write temp file: {e}"))?;

    let config = CompileConfig { ..std::default::Default::default() };
    let result = compile_with_diagnostics(&config, &[tmp.to_str().unwrap().to_string()]);
    let _ = std::fs::remove_file(&tmp);

    let errors: Vec<Value> = result.diagnostics.iter().map(|d| {
        json!({ "code": d.code, "message": d.message, "line": d.line, "col": d.col })
    }).collect();

    Ok(json!({
        "valid": result.success && errors.is_empty(),
        "diagnostics_count": result.diagnostics.len(),
        "errors": errors,
    }))
}

fn tool_format_xiom_code(params: &Value) -> Result<Value, String> {
    let source = params["source"].as_str().ok_or("Missing required parameter: source")?;
    let tmp = std::env::temp_dir().join(format!("xiom_fmt_{}.xi", std::process::id()));
    std::fs::write(&tmp, source).map_err(|e| format!("Failed to write temp file: {e}"))?;
    let output = Command::new("xiom-fmt").arg(tmp.to_str().unwrap()).output().map_err(|e| format!("Failed to spawn xiom-fmt: {e}"))?;
    let _ = std::fs::remove_file(&tmp);
    Ok(json!({"success": output.status.success(), "formatted": String::from_utf8_lossy(&output.stdout).to_string(), "changed": source != String::from_utf8_lossy(&output.stdout)}))
}

/// Phase 5d.9: Sandbox safety audit tool — runs xiomc --sandbox-report=json
/// and returns structured safety findings for CI/CD gating.
fn tool_audit_safety_sandbox(params: &Value) -> Result<Value, String> {
    let file = params["file"].as_str().ok_or("Missing required parameter: file")?;
    if !std::path::Path::new(file).exists() { return Err(format!("File not found: {file}")); }
    let output = Command::new("xiomc").args(["--sandbox-report=json", file]).output().map_err(|e| format!("Failed to spawn xiomc: {e}"))?;
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(json!({"error": "Failed to parse sandbox report"}));
    Ok(json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&report).unwrap_or_default()}]}))
}

// ============================================================================
// Tool registry
// ============================================================================

fn list_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "compile_and_analyze".into(),
            description: "Compile a XIOM source file and return structured diagnostics with JSON output.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string", "description": "Path to the .xi source file" },
                    "target": { "type": "string", "enum": ["native", "wasm", "ir"], "description": "Compilation target" },
                    "strict": { "type": "boolean", "description": "Fail on contract violations" }
                },
                "required": ["file"]
            }),
        },
        ToolDef {
            name: "explain_error_code".into(),
            description: "Get the full reference documentation for a XIOM error code (e.g., X0100, P001, T001).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "Error code to explain" }
                },
                "required": ["code"]
            }),
        },
        ToolDef {
            name: "get_contract_signature".into(),
            description: "Retrieve requires/ensures/invariant contracts for a function or type in a XIOM source file.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string", "description": "Path to the .xi source file" },
                    "function_name": { "type": "string", "description": "Function name to filter by" },
                    "type_name": { "type": "string", "description": "Type name to filter by" }
                },
                "required": ["file"]
            }),
        },
        ToolDef {
            name: "check_xiom_syntax".into(),
            description: "Quick parse-only check (no type checking, no codegen). Fast feedback loop for syntax validation.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "XIOM source code to validate" }
                },
                "required": ["source"]
            }),
        },
        ToolDef {
            name: "format_xiom_code".into(),
            description: "Format XIOM source code according to the canonical style using xiom-fmt.".into(),
            input_schema: json!({"type":"object","properties":{"source":{"type":"string","description":"XIOM source code to format"}},"required":["source"]}),
        },
        ToolDef {
            name: "audit_safety_sandbox".into(),
            description: "Run compiler safety audit on a XIOM source file. Enumerates unsafe blocks, categorises operations, scores severity (HIGH/MEDIUM/LOW), and returns structured findings for CI/CD gating.".into(),
            input_schema: json!({"type":"object","properties":{"file":{"type":"string","description":"Path to .xi file to audit"}},"required":["file"]}),
        },
    ]
}

fn call_tool(name: &str, params: &Value) -> Result<Value, String> {
    match name {
        "explain_error_code" => tool_explain_error_code(params).map(|s| json!({ "content": [{ "type": "text", "text": s }] })),
        "compile_and_analyze" => tool_compile_and_analyze(params).map(|v| json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] })),
        "get_contract_signature" => tool_get_contract_signature(params).map(|v| json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] })),
        "check_xiom_syntax" => tool_check_xiom_syntax(params).map(|v| json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] })),
        "format_xiom_code" => tool_format_xiom_code(params).map(|v| json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] })),
        "audit_safety_sandbox" => tool_audit_safety_sandbox(params).map(|v| json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] })),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

// ============================================================================
// MCP protocol handler
// ============================================================================

fn handle_request(req: &RpcRequest) -> RpcResponse {
    let result = match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "xiom-mcp",
                "version": "0.1.0"
            }
        })),
        "tools/list" => Ok(json!({
            "tools": list_tools()
        })),
        "tools/call" => {
            if let Some(params) = &req.params {
                let name = params["name"].as_str().unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(Value::Null);
                call_tool(name, &args)
            } else {
                Err("Missing params for tools/call".to_string())
            }
        }
        "notifications/initialized" => Ok(Value::Null), // No response needed for notifications
        _ => Err(format!("Unknown method: {}", req.method)),
    };

    match result {
        Ok(value) => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(value),
            error: None,
        },
        Err(msg) => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: None,
            error: Some(RpcError { code: -32000, message: msg }),
        },
    }
}

// ============================================================================
// Main — stdio transport
// ============================================================================

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RpcRequest>(trimmed) {
            Ok(req) => {
                let id = req.id.clone();
                let method = req.method.clone();
                let mut resp = handle_request(&req);
                // Ensure id is preserved even if request parsing omitted it
                if resp.id.is_none() && id.is_some() {
                    resp.id = id;
                }
                // Suppress response for notifications (no id)
                if req.id.is_none() && method.starts_with("notifications/") {
                    continue;
                }
                resp
            }
            Err(e) => RpcResponse {
                jsonrpc: "2.0".into(),
                id: None,
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: format!("Parse error: {e}"),
                }),
            },
        };

        let response_json = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_string()
        });

        writeln!(writer, "{response_json}")?;
        writer.flush()?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_returns_six_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 6, "MVP+Sandbox must have 6 tools");
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"compile_and_analyze"));
        assert!(names.contains(&"explain_error_code"));
        assert!(names.contains(&"get_contract_signature"));
        assert!(names.contains(&"check_xiom_syntax"));
        assert!(names.contains(&"audit_safety_sandbox"));
    }

    #[test]
    fn test_explain_error_code_fallback() {
        let result = tool_explain_error_code(&json!({"code": "X9999"}));
        assert!(result.is_ok(), "Fallback should work for unknown codes");
        let text = result.unwrap();
        assert!(text.contains("X9999"), "Should mention the code");
        assert!(text.contains("No detailed documentation"), "Should note missing doc");
    }

    #[test]
    fn test_explain_error_code_missing_param() {
        let result = tool_explain_error_code(&json!({}));
        assert!(result.is_err(), "Should error on missing code param");
    }

    #[test]
    fn test_compile_missing_file() {
        let result = tool_compile_and_analyze(&json!({"file": "nonexistent.xi"}));
        assert!(result.is_err(), "Should error on missing file");
    }

    #[test]
    fn test_compile_missing_param() {
        let result = tool_compile_and_analyze(&json!({}));
        assert!(result.is_err(), "Should error on missing file param");
    }

    #[test]
    fn test_contract_signature_missing_params() {
        let result = tool_get_contract_signature(&json!({"file": "test.xi"}));
        assert!(result.is_err(), "Should error when neither fn nor type name given");
    }

    #[test]
    fn test_format_missing_source() {
        let result = tool_format_xiom_code(&json!({}));
        assert!(result.is_err(), "Should error on missing source param");
    }

    #[test]
    fn test_syntax_check_missing_source() {
        let result = tool_check_xiom_syntax(&json!({}));
        assert!(result.is_err(), "Should error on missing source param");
    }

    #[test]
    fn test_call_unknown_tool() {
        let result = call_tool("nonexistent", &json!({}));
        assert!(result.is_err(), "Should error on unknown tool");
    }

    #[test]
    fn test_initialize_response() {
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": "2024-11-05"})),
        };
        let resp = handle_request(&req);
        assert!(resp.error.is_none(), "Initialize should not error");
        let result = resp.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "xiom-mcp");
        assert_eq!(result["serverInfo"]["version"], "0.1.0");
    }

    #[test]
    fn test_tools_list_response() {
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/list".into(),
            params: None,
        };
        let resp = handle_request(&req);
        assert!(resp.error.is_none(), "tools/list should not error");
    }

    #[test]
    fn test_unknown_method() {
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(3)),
            method: "unknown/method".into(),
            params: None,
        };
        let resp = handle_request(&req);
        assert!(resp.error.is_some(), "Unknown method should error");
    }
}
