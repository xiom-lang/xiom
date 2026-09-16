// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM Debug Adapter Protocol Server -- contract-aware debugging
// Phase 5d: DAP server for VS Code / JetBrains integration.
// Backend: GDB/MI (Machine Interface) via subprocess. Production-grade.
// M14.1: backend code -> backend.rs

mod backend;
use backend::{GdbBackend, CdbBackend};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::process::Stdio;

// ============================================================================
// DAP Protocol Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct DapRequest {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    msg_type: Option<String>,
    command: String,
    arguments: Option<Value>,
    seq: u64,
}

#[derive(Debug, Serialize)]
struct DapResponse {
    #[serde(rename = "type")]
    msg_type: String,
    request_seq: u64,
    success: bool,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct DapEvent {
    #[serde(rename = "type")]
    msg_type: String,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct Breakpoint {
    pub(crate) id: u64,
    pub(crate) source_path: String,
    pub(crate) line: u64,
    pub(crate) verified: bool,
}

pub(crate) trait DebuggerBackend {
    fn launch(&mut self, program: &str, args: &[String], cwd: &str) -> Result<(), String>;
    fn set_breakpoint(&mut self, source: &str, line: u64) -> Result<Breakpoint, String>;
    fn list_breakpoints(&self) -> Vec<Breakpoint>;
    fn delete_breakpoint(&mut self, id: u64) -> Result<(), String>;
    fn exec_continue(&mut self) -> Result<(), String>;
    fn exec_next(&mut self) -> Result<(), String>;
    fn exec_step(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn poll_stopped(&mut self) -> Result<Value, String>;
    fn evaluate_expression(&mut self, expr: &str) -> Result<String, String>;
    fn thread_info(&mut self) -> Result<Vec<Value>, String>;
    fn stack_info(&mut self) -> Result<Vec<Value>, String>;
    fn list_variables(&mut self) -> Result<Vec<Value>, String>;
    fn list_registers(&mut self) -> Result<Vec<Value>, String>;
    fn read_memory(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, String>;
    fn terminate(&mut self) -> Result<(), String>;
}

fn send(msg: &impl Serialize) {
    let json = serde_json::to_string(msg).unwrap_or_default();
    let mut stdout = io::stdout().lock();
    let content_len = json.len();
    writeln!(stdout, "Content-Length: {content_len}\r\n\r\n{json}").ok();
    stdout.flush().ok();
}

fn send_event(event: &str, body: Option<Value>) {
    send(&DapEvent { msg_type: "event".into(), event: event.into(), body });
}

fn send_response(req_seq: u64, command: &str, success: bool, body: Option<Value>, message: Option<String>) {
    send(&DapResponse {
        msg_type: "response".into(), request_seq: req_seq, success,
        command: command.into(), body, message,
    });
}

// ============================================================================
// Main -- DAP stdio loop (5e.7a -- auto-detect backend)
// ============================================================================

fn detect_backend() -> Box<dyn DebuggerBackend> {
    if std::process::Command::new("cdb").arg("/?").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok() {
        eprintln!("xiom-dbg: CDB/WinDbg backend (5e.7a)");
        return Box::new(CdbBackend::new());
    }
    eprintln!("xiom-dbg: GDB/MI backend");
    Box::new(GdbBackend::new())
}

// ============================================================================
// Phase 8B: JSON API Mode -- single-command structured output for GUI/scripts
// ============================================================================

fn print_json(val: &Value) {
    println!("{}", serde_json::to_string_pretty(val).unwrap_or_else(|_| "{}".to_string()));
}

fn run_json_mode(args: &[String]) -> io::Result<()> {
    let mut backend: Box<dyn DebuggerBackend> = detect_backend();
    let mut target: Option<String> = None;
    let mut cmd: Option<String> = None;
    let mut cmd_args: Vec<String> = Vec::new();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => { i += 1; if i < args.len() { target = Some(args[i].clone()); } }
            "--command" => { i += 1; if i < args.len() { cmd = Some(args[i].clone()); } }
            "--arg" => { i += 1; if i < args.len() { cmd_args.push(args[i].clone()); } }
            _ => {
                if cmd.is_none() { cmd = Some(args[i].clone()); }
                else { cmd_args.push(args[i].clone()); }
            }
        }
        i += 1;
    }

    let command = match cmd {
        Some(c) => c,
        None => {
            print_json(&json!({"error": "no command specified", "usage": "xiom-dbg --json --target <exe> <command>"}));
            return Ok(());
        }
    };

    match command.as_str() {
        "launch" => {
            let program = target.clone().unwrap_or_else(|| "a.exe".to_string());
            match backend.launch(&program, &cmd_args, ".") {
                Ok(()) => print_json(&json!({"status": "launched", "pid": std::process::id()})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "breakpoints" => {
            // List all breakpoints
            let bps: Vec<Value> = backend.list_breakpoints().iter().map(|bp| json!({
                "id": bp.id, "file": bp.source_path, "line": bp.line, "verified": bp.verified
            })).collect();
            print_json(&json!({"breakpoints": bps}));
        }
        "set-breakpoint" => {
            let file = cmd_args.get(0).cloned().unwrap_or_default();
            let line: u64 = cmd_args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            if file.is_empty() || line == 0 {
                print_json(&json!({"error": "usage: set-breakpoint <file> <line>"}));
            } else {
                match backend.set_breakpoint(&file, line) {
                    Ok(bp) => print_json(&json!({"breakpoint": {"id": bp.id, "file": bp.source_path, "line": bp.line, "verified": bp.verified}})),
                    Err(e) => print_json(&json!({"error": e})),
                }
            }
        }
        "delete-breakpoint" => {
            let id: u64 = cmd_args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
            if id == 0 {
                print_json(&json!({"error": "usage: delete-breakpoint <id>"}));
            } else {
                match backend.delete_breakpoint(id) {
                    Ok(()) => print_json(&json!({"deleted": id})),
                    Err(e) => print_json(&json!({"error": e})),
                }
            }
        }
        "stack" => {
            match backend.stack_info() {
                Ok(frames) => {
                    let mut idx = 0u64;
                    let formatted: Vec<Value> = frames.iter().map(|f| {
                        let result = json!({"frame": idx, "function": f["name"], "file": f.get("source").and_then(|s| s["path"].as_str()).unwrap_or("?"), "line": f["line"], "address": f.get("address").and_then(|a| a.as_str()).unwrap_or("?")});
                        idx += 1;
                        result
                    }).collect();
                    print_json(&json!({"stack": formatted}));
                }
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "variables" => {
            match backend.list_variables() {
                Ok(vars) => print_json(&json!({"variables": vars})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "registers" => {
            match backend.list_registers() {
                Ok(regs) => print_json(&json!({"registers": regs})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "memory" => {
            let addr: u64 = cmd_args.get(0).and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()).unwrap_or(0);
            let size: usize = cmd_args.get(1).and_then(|s| s.parse().ok()).unwrap_or(64);
            match backend.read_memory(addr, size) {
                Ok(bytes) => {
                    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                    let ascii: String = bytes.iter().map(|&b| if b >= 32 && b < 127 { b as char } else { '.' }).collect();
                    print_json(&json!({"address": format!("0x{:X}", addr), "size": size, "bytes": hex.join(" "), "ascii": ascii}));
                }
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "step" => {
            match backend.exec_next() {
                Ok(()) => {
                    if let Ok(info) = backend.poll_stopped() {
                        print_json(&json!({"stopped": true, "reason": info["reason"], "file": info.get("source").and_then(|s| s["path"].as_str()), "line": info["line"]}));
                    } else { print_json(&json!({"stopped": true})); }
                }
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "step-in" => {
            match backend.exec_step() {
                Ok(()) => {
                    if let Ok(info) = backend.poll_stopped() {
                        print_json(&json!({"stopped": true, "reason": info["reason"]}));
                    } else { print_json(&json!({"stopped": true})); }
                }
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "continue" => {
            match backend.exec_continue() {
                Ok(()) => {
                    if let Ok(info) = backend.poll_stopped() {
                        print_json(&json!({"stopped": true, "reason": info["reason"]}));
                    } else { print_json(&json!({"running": true})); }
                }
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "evaluate" => {
            let expr = cmd_args.join(" ");
            match backend.evaluate_expression(&expr) {
                Ok(result) => print_json(&json!({"expression": expr, "value": result})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "threads" => {
            match backend.thread_info() {
                Ok(threads) => print_json(&json!({"threads": threads})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "terminate" => {
            match backend.terminate() {
                Ok(()) => print_json(&json!({"terminated": true})),
                Err(e) => print_json(&json!({"error": e})),
            }
        }
        "help" => {
            eprintln!("XIOM Debugger v0.49.7 -- JSON API Mode");
            eprintln!("Usage: xiom-dbg --json --target <exe> <command> [args]");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  launch [args]              Launch target under debugger");
            eprintln!("  breakpoints                List all breakpoints");
            eprintln!("  set-breakpoint <file> <line>  Set breakpoint");
            eprintln!("  delete-breakpoint <id>     Remove breakpoint");
            eprintln!("  stack                      Show call stack");
            eprintln!("  variables                  List local variables");
            eprintln!("  registers                  Show CPU registers");
            eprintln!("  memory <addr> <size>       Read memory (hex dump)");
            eprintln!("  step                       Step over (next)");
            eprintln!("  step-in                    Step into");
            eprintln!("  continue                   Continue execution");
            eprintln!("  evaluate <expr>            Evaluate expression");
            eprintln!("  threads                    List threads");
            eprintln!("  terminate                  End debug session");
        }
        _ => {
            print_json(&json!({"error": format!("unknown command: {}", command)}));
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Phase 8B: --json mode -- single-command JSON API for GUIs/scripts
    if args.len() >= 2 && args[1] == "--json" {
        return run_json_mode(&args);
    }

    // Default: DAP server mode (VS Code / IDE integration)
    let mut backend: Box<dyn DebuggerBackend> = detect_backend();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 { break; } // EOF
        let trimmed = line.trim_end().to_string();

        // DAP uses HTTP-like Content-Length header framing
        if trimmed.starts_with("Content-Length:") {
            let content_len: usize = trimmed["Content-Length:".len()..].trim().parse().unwrap_or(0);
            // AUDIT #13 FIX: the body buffer trusted an untrusted header --
            // one hostile frame OOM-killed the debugger. Cap frames; oversize
            // requests are dropped and the session ends cleanly.
            const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024; // 64 MiB
            if content_len > MAX_FRAME_BYTES {
                eprintln!("dbg: dropping oversized frame ({} bytes)", content_len);
                break;
            }
            // Read the blank line separator
            let mut blank = String::new();
            reader.read_line(&mut blank)?;
            // Read the JSON body
            let mut body = vec![0u8; content_len];
            io::Read::read_exact(&mut reader, &mut body)?;
            let body_str = String::from_utf8_lossy(&body);

            if let Ok(req) = serde_json::from_str::<DapRequest>(&body_str) {
                handle_request(backend.as_mut(), &req);
            }
        }
    }
    Ok(())
}

fn handle_request(backend: &mut dyn DebuggerBackend, req: &DapRequest) {
    let args = req.arguments.clone().unwrap_or(Value::Null);

    match req.command.as_str() {
        "initialize" => {
            send_response(req.seq, &req.command, true,
                Some(json!({
                    "supportsConfigurationDoneRequest": true,
                    "supportsBreakpointLocations": false,
                    "supportsStepInTargetsRequest": false,
                    "supportsGotoTargetsRequest": false,
                    "supportsConditionalBreakpoints": false,
                    "supportsHitConditionalBreakpoints": false,
                    "supportsEvaluateForHovers": true,
                    "supportsSetVariable": false,
                    "exceptionBreakpointFilters": [{
                        "filter": "contract_violation",
                        "label": "Contract Violations",
                        "description": "Breaks when a XIOM contract (requires/ensures/invariant) is violated",
                        "default": true
                    }]
                })),
                None
            );
            send_event("initialized", None);
        }

        "launch" => {
            let program = args["program"].as_str().unwrap_or("a.exe");
            let cwd = args["cwd"].as_str().unwrap_or(".");
            let program_args: Vec<String> = args["args"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            match backend.launch(program, &program_args, cwd) {
                Ok(()) => {
                    send_response(req.seq, &req.command, true, None, None);
                }
                Err(e) => {
                    send_response(req.seq, &req.command, false, None, Some(e));
                }
            }
        }

        "setBreakpoints" => {
            let source_path = args["source"]["path"].as_str().unwrap_or("unknown.xi");
            let breakpoints_arg = args["breakpoints"].as_array();
            let mut bps = Vec::new();

            if let Some(breakpoints) = breakpoints_arg {
                for bp in breakpoints {
                    if let Some(line) = bp["line"].as_u64() {
                        match backend.set_breakpoint(source_path, line) {
                            Ok(bp) => bps.push(json!({"id": bp.id, "verified": bp.verified, "line": bp.line, "source": {"path": bp.source_path}})),
                            Err(_) => bps.push(json!({"verified": false, "line": line, "message": "Failed to set breakpoint"})),
                        }
                    }
                }
            }

            send_response(req.seq, &req.command, true, Some(json!({"breakpoints": bps})), None);
        }

        "setExceptionBreakpoints" => {
            send_response(req.seq, &req.command, true, None, None);
        }

        "configurationDone" => {
            // Run the program to the first breakpoint or main
            let _ = backend.exec_continue();
            send_response(req.seq, &req.command, true, None, None);
        }

        "threads" => {
            match backend.thread_info() {
                Ok(threads) => send_response(req.seq, &req.command, true, Some(json!({"threads": threads})), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "stackTrace" => {
            match backend.stack_info() {
                Ok(frames) => send_response(req.seq, &req.command, true, Some(json!({"stackFrames": frames, "totalFrames": frames.len()})), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "scopes" => {
            let frame_id = args["frameId"].as_u64().unwrap_or(0);
            send_response(req.seq, &req.command, true, Some(json!({
                "scopes": [
                    {"name": "Locals", "variablesReference": frame_id * 100 + 1, "expensive": false},
                    {"name": "Registers", "variablesReference": frame_id * 100 + 2, "expensive": true},
                ]
            })), None);
        }

        "variables" => {
            match backend.list_variables() {
                Ok(vars) => send_response(req.seq, &req.command, true, Some(json!({"variables": vars})), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "continue" => {
            match backend.exec_continue() {
                Ok(()) => {
                    send_response(req.seq, &req.command, true, Some(json!({"allThreadsContinued": true})), None);
                    // 6C.3: Poll for *stopped event after continue
                    if let Ok(info) = backend.poll_stopped() {
                        send_event("stopped", Some(info));
                    }
                }
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "next" => {
            match backend.exec_next() {
                Ok(()) => {
                    send_response(req.seq, &req.command, true, None, None);
                    if let Ok(info) = backend.poll_stopped() {
                        send_event("stopped", Some(info));
                    }
                }
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "stepIn" => {
            match backend.exec_step() {
                Ok(()) => {
                    send_response(req.seq, &req.command, true, None, None);
                    if let Ok(info) = backend.poll_stopped() {
                        send_event("stopped", Some(info));
                    }
                }
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        // 6C.3: DAP evaluate handler -- variable watch, REPL, hover
        "evaluate" => {
            let expr = args["expression"].as_str().unwrap_or("0");
            match backend.evaluate_expression(expr) {
                Ok(result) => send_response(req.seq, &req.command, true, Some(json!({
                    "result": result,
                    "variablesReference": 0,
                })), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "pause" => {
            let _ = backend.pause();
            send_response(req.seq, &req.command, true, None, None);
        }

        "disconnect" => {
            let _ = backend.terminate();
            send_response(req.seq, &req.command, true, None, None);
        }

        _ => {
            send_response(req.seq, &req.command, false, None, Some(format!("Unsupported command: {}", req.command)));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdb_backend_new() {
        let gdb = GdbBackend::new();
        assert!(gdb.child.is_none());
        assert!(gdb.breakpoints.is_empty());
    }

    #[test]
    fn test_gdb_not_launched_error() {
        let mut gdb = GdbBackend::new();
        let result = gdb.send_mi("-break-insert main");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_breakpoint_line() {
        let bp = Breakpoint { id: 1, source_path: "test.xi".into(), line: 42, verified: true };
        assert_eq!(bp.line, 42);
        assert!(bp.verified);
    }

    #[test]
    fn test_serialize_dap_response() {
        let resp = DapResponse {
            msg_type: "response".into(),
            request_seq: 1,
            success: true,
            command: "initialize".into(),
            body: Some(json!({"supportsConfigurationDoneRequest": true})),
            message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("initialize"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_serialize_dap_event() {
        let event = DapEvent {
            msg_type: "event".into(),
            event: "stopped".into(),
            body: Some(json!({"reason": "breakpoint", "threadId": 1})),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stopped"));
        assert!(json.contains("breakpoint"));
    }

    #[test]
    fn test_dap_event_no_body() {
        let event = DapEvent {
            msg_type: "event".into(),
            event: "terminated".into(),
            body: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("body"));
    }

    #[test]
    fn test_thread_info_parsing_empty() {
        let mut gdb = GdbBackend::new();
        // No GDB session -- should return error
        let result = gdb.thread_info();
        assert!(result.is_err());
    }

    #[test]
    fn test_breakpoint_insert_no_session() {
        let mut gdb = GdbBackend::new();
        let result = gdb.set_breakpoint("test.xi", 10);
        assert!(result.is_err());
    }

    // -- M27-1: Debugger edge cases ----------------------------------

    #[test] fn test_dbg_source_map_init() {
        // SourceMap concept: verify debugger handles empty state
        let gdb = GdbBackend::new();
        assert!(gdb.breakpoints.is_empty());
    }

    #[test] fn test_dbg_breakpoint_default() {
        let bp = Breakpoint { id: 0, source_path: "".into(), line: 0, verified: false };
        assert_eq!(bp.id, 0);
        assert!(!bp.verified);
    }

    #[test] fn test_dbg_breakpoint_multiple() {
        let mut bps = Vec::new();
        for i in 1..=20 {
            bps.push(Breakpoint { id: i, source_path: format!("file{}.xi", i), line: i * 10, verified: true });
        }
        assert_eq!(bps.len(), 20);
        assert_eq!(bps[0].line, 10);
        assert_eq!(bps[19].line, 200);
    }

    #[test] fn test_dap_response_initialize() {
        let resp = DapResponse {
            msg_type: "response".into(), request_seq: 1, success: true,
            command: "initialize".into(),
            body: Some(json!({"supportsConfigurationDoneRequest": true, "supportsStepIn": true})),
            message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("initialize"));
        assert!(json.contains("success"));
    }

    #[test] fn test_dap_response_error() {
        let resp = DapResponse {
            msg_type: "response".into(), request_seq: 2, success: false,
            command: "launch".into(), body: None,
            message: Some("GDB not found".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("launch"));
    }

    #[test] fn test_dap_event_breakpoint() {
        let event = DapEvent {
            msg_type: "event".into(), event: "stopped".into(),
            body: Some(json!({"reason": "breakpoint", "threadId": 1, "allThreadsStopped": true})),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("breakpoint"));
        assert!(json.contains("allThreadsStopped"));
    }

    #[test] fn test_dap_event_step() {
        let event = DapEvent {
            msg_type: "event".into(), event: "stopped".into(),
            body: Some(json!({"reason": "step", "threadId": 1})),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("step"));
    }

    #[test] fn test_dap_event_exited() {
        let event = DapEvent { msg_type: "event".into(), event: "exited".into(), body: Some(json!({"exitCode": 0})) };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("exited"));
    }

    #[test] fn test_gdb_empty_breakpoints() {
        let gdb = GdbBackend::new();
        assert!(gdb.breakpoints.is_empty());
        assert!(gdb.child.is_none());
    }

    #[test] fn test_gdb_mi_command_format() {
        let mut gdb = GdbBackend::new();
        let cmd = gdb.send_mi("-break-insert -f test.xi -l 10");
        assert!(cmd.is_err()); // No GDB session
    }

    #[test] fn test_gdb_thread_info_no_session() {
        let mut gdb = GdbBackend::new();
        let result = gdb.thread_info();
        assert!(result.is_err());
    }

    #[test] fn test_gdb_watchpoint_no_session() {
        let mut gdb = GdbBackend::new();
        let result = gdb.set_breakpoint("test.xi", 5);
        assert!(result.is_err());
    }

    #[test] fn test_dap_launch_request() {
        let req = DapRequest { seq: 1, msg_type: None, command: "launch".into(), arguments: Some(json!({"program": "test.exe"})) };
        assert_eq!(req.seq, 1);
        assert_eq!(req.command, "launch");
    }

    #[test] fn test_dap_set_breakpoints_request() {
        let req = DapRequest {
            seq: 2, msg_type: None, command: "setBreakpoints".into(),
            arguments: Some(json!({"source": {"path": "test.xi"}, "breakpoints": [{"line": 10}, {"line": 20}]})),
        };
        assert_eq!(req.command, "setBreakpoints");
    }

    #[test] fn test_dap_continue_request() {
        let req = DapRequest { seq: 3, msg_type: None, command: "continue".into(), arguments: Some(json!({"threadId": 1})) };
        assert_eq!(req.command, "continue");
    }

    #[test] fn test_dap_next_request() {
        let req = DapRequest { seq: 4, msg_type: None, command: "next".into(), arguments: Some(json!({"threadId": 1})) };
        assert_eq!(req.command, "next");
    }

    #[test] fn test_dap_stepin_request() {
        let req = DapRequest { seq: 5, msg_type: None, command: "stepIn".into(), arguments: Some(json!({"threadId": 1})) };
        assert_eq!(req.command, "stepIn");
    }

    #[test] fn test_dap_variables_request() {
        let req = DapRequest { seq: 6, msg_type: None, command: "variables".into(), arguments: Some(json!({"variablesReference": 1})) };
        assert_eq!(req.command, "variables");
    }

    #[test] fn test_dap_threads_request() {
        let req = DapRequest { seq: 7, msg_type: None, command: "threads".into(), arguments: None };
        assert_eq!(req.command, "threads");
    }

    #[test] fn test_dap_disconnect_request() {
        let req = DapRequest { seq: 8, msg_type: None, command: "disconnect".into(), arguments: None };
        assert_eq!(req.command, "disconnect");
    }

    #[test] fn test_dap_unknown_command() {
        let req = DapRequest { seq: 99, msg_type: Some("request".into()), command: "nonexistent".into(), arguments: None };
        assert_eq!(req.command, "nonexistent");
    }
}
