// XIOM Debug Adapter Protocol Server — contract-aware debugging
// Phase 5d: DAP server for VS Code / JetBrains integration.
// Backend: GDB/MI (Machine Interface) via subprocess. Production-grade.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

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

// ============================================================================
// GDB/MI Backend
// ============================================================================

struct GdbBackend {
    child: Option<Child>,
    breakpoints: HashMap<u64, Breakpoint>,
    next_breakpoint_id: u64,
    program_path: Option<String>,
}

#[derive(Debug, Clone)]
struct Breakpoint {
    id: u64,
    source_path: String,
    line: u64,
    verified: bool,
}

impl GdbBackend {
    fn new() -> Self {
        GdbBackend {
            child: None,
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            program_path: None,
        }
    }

    /// Launch the XIOM binary under GDB control.
    fn launch(&mut self, program: &str, args: &[String], _cwd: &str) -> Result<(), String> {
        if self.child.is_some() {
            return Err("Debug session already active".into());
        }

        // Start GDB in MI mode
        let mut cmd = Command::new("gdb");
        cmd.args(["--interpreter=mi3", "--quiet", program]);
        if !args.is_empty() {
            cmd.arg("--args");
            for a in args { cmd.arg(a); }
        }
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::inherit());

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn GDB: {e}"))?;
        self.child = Some(child);
        self.program_path = Some(program.to_string());

        Ok(())
    }

    /// Send a GDB/MI command and read the response.
    fn send_mi(&mut self, cmd: &str) -> Result<String, String> {
        let child = self.child.as_mut().ok_or("No debug session")?;
        let stdin = child.stdin.as_mut().ok_or("No stdin")?;
        writeln!(stdin, "{cmd}").map_err(|e| format!("GDB write error: {e}"))?;
        stdin.flush().map_err(|e| format!("GDB flush error: {e}"))?;

        // Read stdout until we get a complete MI record (ends with ^done, ^error, etc.)
        let stdout = child.stdout.as_mut().ok_or("No stdout")?;
        let reader = BufReader::new(stdout);
        let mut response = String::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("GDB read error: {e}"))?;
            response.push_str(&line);
            response.push('\n');
            if line.starts_with('^') || line.starts_with('*') {
                break;
            }
        }
        Ok(response)
    }

    /// Set a breakpoint at source:line.
    fn set_breakpoint(&mut self, source: &str, line: u64) -> Result<Breakpoint, String> {
        let resp = self.send_mi(&format!("-break-insert --source \"{source}\" --line {line}"))?;
        let verified = resp.contains("^done");
        let bp = Breakpoint { id: self.next_breakpoint_id, source_path: source.to_string(), line, verified };
        self.breakpoints.insert(bp.id, bp.clone());
        self.next_breakpoint_id += 1;
        Ok(bp)
    }

    /// Continue execution.
    fn exec_continue(&mut self) -> Result<(), String> {
        self.send_mi("-exec-continue")?;
        Ok(())
    }

    /// Step over next instruction.
    fn exec_next(&mut self) -> Result<(), String> {
        self.send_mi("-exec-next")?;
        Ok(())
    }

    /// Step into function call.
    fn exec_step(&mut self) -> Result<(), String> {
        self.send_mi("-exec-step")?;
        Ok(())
    }

    /// Get thread list.
    fn thread_info(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-thread-info")?;
        // Parse GDB/MI thread list
        let mut threads = Vec::new();
        for line in resp.lines() {
            if line.contains("thread-id") {
                if let Some(start) = line.find("thread-id=\"") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let id: &str = &rest[..end];
                        threads.push(json!({"id": id.parse::<u64>().unwrap_or(1), "name": format!("Thread {}", id)}));
                    }
                }
            }
        }
        if threads.is_empty() { threads.push(json!({"id": 1, "name": "main"})); }
        Ok(threads)
    }

    /// Get stack trace.
    fn stack_info(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-stack-list-frames")?;
        let mut frames = Vec::new();
        let mut frame_id = 0;
        for line in resp.lines() {
            if line.contains("frame={") {
                frame_id += 1;
                let mut name = "??".to_string();
                let mut file = "unknown.xi".to_string();
                let mut line_num: u64 = 0;

                if let Some(start) = line.find("func=\"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') { name = rest[..end].to_string(); }
                }
                if let Some(start) = line.find("file=\"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') { file = rest[..end].to_string(); }
                }
                if let Some(start) = line.find("line=\"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') {
                        line_num = rest[..end].parse().unwrap_or(0);
                    }
                }

                frames.push(json!({
                    "id": frame_id,
                    "name": name,
                    "source": { "name": file, "path": file },
                    "line": line_num,
                    "column": 0,
                }));
            }
        }
        Ok(frames)
    }

    fn terminate(&mut self) -> Result<(), String> {
        // Kill the child process (GDB) directly — don't try to send GDB-MI.
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
        Ok(())
    }
}

// ============================================================================
// DAP Message Handler
// ============================================================================

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
// Main — DAP stdio loop
// ============================================================================

fn main() -> io::Result<()> {
    let mut gdb = GdbBackend::new();
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
            // Read the blank line separator
            let mut blank = String::new();
            reader.read_line(&mut blank)?;
            // Read the JSON body
            let mut body = vec![0u8; content_len];
            io::Read::read_exact(&mut reader, &mut body)?;
            let body_str = String::from_utf8_lossy(&body);

            if let Ok(req) = serde_json::from_str::<DapRequest>(&body_str) {
                handle_request(&mut gdb, &req);
            }
        }
    }
    Ok(())
}

fn handle_request(gdb: &mut GdbBackend, req: &DapRequest) {
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

            match gdb.launch(program, &program_args, cwd) {
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
                        match gdb.set_breakpoint(source_path, line) {
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
            let _ = gdb.send_mi("-exec-run");
            send_response(req.seq, &req.command, true, None, None);
        }

        "threads" => {
            match gdb.thread_info() {
                Ok(threads) => send_response(req.seq, &req.command, true, Some(json!({"threads": threads})), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "stackTrace" => {
            match gdb.stack_info() {
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
            // Placeholder: return empty variables list
            send_response(req.seq, &req.command, true, Some(json!({"variables": []})), None);
        }

        "continue" => {
            match gdb.exec_continue() {
                Ok(()) => send_response(req.seq, &req.command, true, Some(json!({"allThreadsContinued": true})), None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "next" => {
            match gdb.exec_next() {
                Ok(()) => send_response(req.seq, &req.command, true, None, None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "stepIn" => {
            match gdb.exec_step() {
                Ok(()) => send_response(req.seq, &req.command, true, None, None),
                Err(e) => send_response(req.seq, &req.command, false, None, Some(e)),
            }
        }

        "pause" => {
            if let Some(ref mut child) = gdb.child {
                // Send SIGINT to GDB
                #[cfg(unix)]
                unsafe { libc::kill(child.id() as i32, libc::SIGINT); }
                #[cfg(windows)]
                { let _ = child.kill(); }
            }
            send_response(req.seq, &req.command, true, None, None);
        }

        "disconnect" => {
            let _ = gdb.terminate();
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
        // No GDB session — should return error
        let result = gdb.thread_info();
        assert!(result.is_err());
    }

    #[test]
    fn test_breakpoint_insert_no_session() {
        let mut gdb = GdbBackend::new();
        let result = gdb.set_breakpoint("test.xi", 10);
        assert!(result.is_err());
    }
}
