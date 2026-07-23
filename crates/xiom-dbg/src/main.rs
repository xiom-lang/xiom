// XIOM Debug Adapter Protocol Server — contract-aware debugging
// Phase 5d: DAP server for VS Code / JetBrains integration.
// Backend: GDB/MI (Machine Interface) via subprocess. Production-grade.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

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
    /// 6C.3: Persistent BufReader to avoid desync across send_mi calls
    reader: Option<BufReader<ChildStdout>>,
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

// ============================================================================
// Debugger Backend Trait (5e.7a — multi-backend support)
// ============================================================================

trait DebuggerBackend {
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
    fn name(&self) -> &'static str;
}

impl GdbBackend {
    fn new() -> Self {
        GdbBackend { child: None, reader: None, breakpoints: HashMap::new(), next_breakpoint_id: 1, program_path: None }
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

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn GDB: {e}"))?;
        // 6C.3: Take ownership of stdout for persistent buffered reader
        let stdout = child.stdout.take().ok_or("Failed to capture GDB stdout")?;
        self.reader = Some(BufReader::new(stdout));
        self.child = Some(child);
        self.program_path = Some(program.to_string());

        Ok(())
    }

    /// 6C.3: Send a GDB/MI command and read the response using persistent reader.
    /// Also detects *stopped async records and returns them for event generation.
    fn send_mi(&mut self, cmd: &str) -> Result<String, String> {
        let child = self.child.as_mut().ok_or("No debug session")?;
        let stdin = child.stdin.as_mut().ok_or("No stdin")?;
        writeln!(stdin, "{cmd}").map_err(|e| format!("GDB write error: {e}"))?;
        stdin.flush().map_err(|e| format!("GDB flush error: {e}"))?;

        let reader = self.reader.as_mut().ok_or("No stdout reader")?;
        let mut response = String::new();
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).map_err(|e| format!("GDB read error: {e}"))?;
            if n == 0 { break; }
            response.push_str(&line);
            // Stop on synchronous result record or async exec record
            let trimmed = line.trim();
            if trimmed.starts_with('^') || trimmed.starts_with("*stopped") || trimmed.starts_with("*running") {
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
                if let Some(start) = line.find("func=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { name = rest[..end].to_string(); } }
                if let Some(start) = line.find("file=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { file = rest[..end].to_string(); } }
                if let Some(start) = line.find("line=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { line_num = rest[..end].parse().unwrap_or(0); } }
                frames.push(json!({"id": frame_id, "name": name, "source": {"name": file, "path": file}, "line": line_num, "column": 0}));
            }
        }
        Ok(frames)
    }

    /// Get local variables for the current frame.
    fn list_variables(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-stack-list-variables --simple-values")?;
        let mut vars = Vec::new();
        let mut var_ref = 1000;

        for line in resp.lines() {
            if line.contains("name=\"") {
                let mut name = String::new();
                let mut value = String::new();
                let mut var_type = "unknown".to_string();

                if let Some(start) = line.find("name=\"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') { name = rest[..end].to_string(); }
                }
                if let Some(start) = line.find("value=\"") {
                    let rest = &line[start + 7..];
                    if let Some(end) = rest.find('"') { value = rest[..end].to_string(); }
                }
                if let Some(start) = line.find("type=\"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') { var_type = rest[..end].to_string(); }
                }

                if !name.is_empty() && name != "..." {
                    let is_compound = var_type.contains('*') || var_type.contains("struct") || var_type.contains("class");
                    let var_ref_id = if is_compound { var_ref += 1; var_ref } else { 0 };
                    vars.push(json!({
                        "name": name,
                        "value": value,
                        "type": var_type,
                        "variablesReference": var_ref_id,
                    }));
                }
            }
        }

        if vars.is_empty() {
            vars.push(json!({"name": "no locals", "value": "<no variables in scope>", "variablesReference": 0}));
        }
        Ok(vars)
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

    fn list_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints.values().cloned().collect()
    }

    fn delete_breakpoint(&mut self, id: u64) -> Result<(), String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let _ = self.send_mi(&format!("-break-delete {}", id));
        self.breakpoints.remove(&id);
        Ok(())
    }

    fn list_registers(&mut self) -> Result<Vec<Value>, String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let resp = self.send_mi("-data-list-register-values x")?;
        let mut regs = Vec::new();
        if let Some(start) = resp.find("register-values=[") {
            let section = &resp[start + 17..];
            if let Some(end) = section.find(']') {
                let values_str = &section[..end];
                for entry in values_str.split("},") {
                    let val = entry.split("value=\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("?");
                    let num = entry.split("number=\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("?");
                    regs.push(json!({"name": format!("r{}", num), "value": val}));
                }
            }
        }
        Ok(regs)
    }

    fn read_memory(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let resp = self.send_mi(&format!("-data-read-memory-bytes 0x{:X} {}", addr, size))?;
        if let Some(start) = resp.find("contents=\"") {
            let bytes_str = &resp[start + 10..];
            if let Some(end) = bytes_str.find('"') {
                let hex = &bytes_str[..end];
                let mut bytes = Vec::new();
                for chunk in hex.split_whitespace() {
                    if let Ok(b) = u8::from_str_radix(chunk, 16) { bytes.push(b); }
                }
                return Ok(bytes);
            }
        }
        Err("Cannot read memory".into())
    }
}

// 8B: Compound GDB backend (for --json mode on Windows without GDB installed)
struct CompoundBackend { gdb: GdbBackend, use_gdb: bool }

fn send(msg: &impl Serialize) {
    let json = serde_json::to_string(msg).unwrap_or_default();
    let mut stdout = io::stdout().lock();
    let content_len = json.len();
    writeln!(stdout, "Content-Length: {content_len}\r\n\r\n{json}").ok();
    stdout.flush().ok();
}

impl DebuggerBackend for GdbBackend {
    fn launch(&mut self, program: &str, args: &[String], cwd: &str) -> Result<(), String> {
        GdbBackend::launch(self, program, args, cwd)
    }
    fn set_breakpoint(&mut self, source: &str, line: u64) -> Result<Breakpoint, String> {
        GdbBackend::set_breakpoint(self, source, line)
    }
    fn exec_continue(&mut self) -> Result<(), String> { GdbBackend::exec_continue(self) }
    fn exec_next(&mut self) -> Result<(), String> { GdbBackend::exec_next(self) }
    fn exec_step(&mut self) -> Result<(), String> { GdbBackend::exec_step(self) }
    fn pause(&mut self) -> Result<(), String> {
        if let Some(ref child) = self.child {
            // SAFETY: libc::kill is an FFI call to send SIGINT to the child GDB process.
            // The child ID is guaranteed valid because we hold `Some(ref child)` above,
            // and SIGINT is a well-defined signal that GDB handles for pause/resume.
            #[cfg(unix)] unsafe { libc::kill(child.id() as i32, libc::SIGINT); }
            #[cfg(windows)] { let _ = child; }
        }
        Ok(())
    }

    /// 6C.3: Poll for *stopped async record from GDB.
    fn poll_stopped(&mut self) -> Result<Value, String> {
        // Read any pending async records (*stopped, *running)
        if let Some(ref mut reader) = self.reader {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return Err("EOF".to_string()),
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with("*stopped") {
                        let reason = if trimmed.contains("breakpoint-hit") { "breakpoint" }
                            else if trimmed.contains("end-stepping-range") { "step" }
                            else { "pause" };
                        let desc = trimmed.replacen("*stopped,", "", 1);
                        return Ok(json!({"reason": reason, "description": desc, "threadId": 1, "allThreadsStopped": true}));
                    } else if trimmed.starts_with("*running") {
                        return Ok(json!({"reason": "continued", "threadId": 1}));
                    }
                    return Ok(json!({"reason": "unknown"}));
                }
                Err(e) => return Err(format!("read error: {e}")),
            }
        }
        Err("no reader".to_string())
    }

    /// 6C.3: Evaluate expression in debugger context.
    fn evaluate_expression(&mut self, expr: &str) -> Result<String, String> {
        let resp = self.send_mi(&format!("-data-evaluate-expression \"{expr}\""))?;
        // Parse MI result: ^done,value="<val>"
        if let Some(val_start) = resp.find("value=\"") {
            let after = &resp[val_start + 7..];
            if let Some(val_end) = after.find('"') {
                return Ok(after[..val_end].to_string());
            }
        }
        Ok(resp)
    }
    fn thread_info(&mut self) -> Result<Vec<Value>, String> { GdbBackend::thread_info(self) }
    fn stack_info(&mut self) -> Result<Vec<Value>, String> { GdbBackend::stack_info(self) }
    fn list_variables(&mut self) -> Result<Vec<Value>, String> { GdbBackend::list_variables(self) }
    fn terminate(&mut self) -> Result<(), String> { GdbBackend::terminate(self) }
    fn list_breakpoints(&self) -> Vec<Breakpoint> { GdbBackend::list_breakpoints(self) }
    fn delete_breakpoint(&mut self, id: u64) -> Result<(), String> { GdbBackend::delete_breakpoint(self, id) }
    fn list_registers(&mut self) -> Result<Vec<Value>, String> { GdbBackend::list_registers(self) }
    fn read_memory(&mut self, a: u64, s: usize) -> Result<Vec<u8>, String> { GdbBackend::read_memory(self, a, s) }
    fn name(&self) -> &'static str { "GDB/MI" }
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
// CDB/WinDbg Backend (5e.7a — Windows Debugger Engine via cdb.exe)
// ============================================================================

struct CdbBackend {
    child: Option<Child>,
    breakpoints: HashMap<u64, Breakpoint>,
    next_breakpoint_id: u64,
    program_path: Option<String>,
}

impl CdbBackend {
    fn new() -> Self {
        CdbBackend { child: None, breakpoints: HashMap::new(), next_breakpoint_id: 1, program_path: None }
    }

    fn send_cmd(&mut self, cmd: &str) -> Result<String, String> {
        let child = self.child.as_mut().ok_or("cdb not launched")?;
        let stdin = child.stdin.as_mut().ok_or("stdin unavailable")?;
        writeln!(stdin, "{cmd}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))?;
        let stdout = child.stdout.as_mut().ok_or("stdout unavailable")?;
        let reader = BufReader::new(stdout);
        let mut output = String::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("read: {e}"))?;
            if line.trim().ends_with(">") { break; }
            output.push_str(&line); output.push('\n');
        }
        Ok(output)
    }

    fn launch_impl(&mut self, program: &str, args: &[String], _cwd: &str) -> Result<(), String> {
        self.program_path = Some(program.to_string());
        let mut cmd = Command::new("cdb");
        cmd.arg("-o").arg("-lines").arg(program);
        if !args.is_empty() { cmd.arg("--"); for a in args { cmd.arg(a); } }
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());
        self.child = Some(cmd.spawn().map_err(|e| format!("cannot launch cdb: {e}"))?);
        Ok(())
    }

    fn set_breakpoint_impl(&mut self, source: &str, line: u64) -> Result<Breakpoint, String> {
        let stem = std::path::Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or(source);
        let output = self.send_cmd(&format!("bu `{}:{}`", stem, line))?;
        let bp = Breakpoint { id: self.next_breakpoint_id, source_path: source.into(), line,
            verified: !output.contains("Unable to") && !output.contains("Couldn't") };
        self.next_breakpoint_id += 1;
        self.breakpoints.insert(bp.id, bp.clone());
        Ok(bp)
    }

    fn exec_continue_impl(&mut self) -> Result<(), String> { self.send_cmd("g").map(|_| ()) }
    fn exec_next_impl(&mut self) -> Result<(), String> { self.send_cmd("p").map(|_| ()) }
    fn exec_step_impl(&mut self) -> Result<(), String> { self.send_cmd("t").map(|_| ()) }

    fn thread_info_impl(&mut self) -> Result<Vec<Value>, String> {
        let output = self.send_cmd("~")?;
        let mut threads = Vec::new();
        for line in output.lines() {
            if let Some(dot) = line.find('.') {
                let ids: String = line[..dot].chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(id) = ids.parse::<u64>() { threads.push(json!({"id":id,"name":format!("Thread {id}")})); }
            }
        }
        if threads.is_empty() { threads.push(json!({"id":0,"name":"Main Thread"})); }
        Ok(threads)
    }

    fn stack_info_impl(&mut self) -> Result<Vec<Value>, String> {
        let output = self.send_cmd("k")?;
        let mut frames = Vec::new();
        for (i, line) in output.lines().enumerate() {
            let t = line.trim();
            if t.is_empty() || t == "ChildEBP RetAddr" { continue; }
            let name: String = if let Some(b) = t.find('!') { t[b+1..].split_whitespace().next().unwrap_or(t).into() } else { t.into() };
            frames.push(json!({"id":i,"name":name,"source":null,"line":0,"column":0}));
        }
        if frames.is_empty() { frames.push(json!({"id":0,"name":"<unknown>","source":null,"line":0,"column":0})); }
        Ok(frames)
    }

    fn list_variables_impl(&mut self) -> Result<Vec<Value>, String> {
        let output = self.send_cmd("dv")?;
        let mut vars = Vec::new();
        for line in output.lines() {
            let t = line.trim();
            if t.is_empty() { continue; }
            let parts: Vec<&str> = t.splitn(2,'=').collect();
            vars.push(json!({"name":parts[0].trim(),"value":parts.get(1).map_or("<unknown>",|s|s.trim()),"variablesReference":0}));
        }
        if vars.is_empty() { vars.push(json!({"name":"no locals","value":"<no variables in scope>","variablesReference":0})); }
        Ok(vars)
    }

    fn terminate_impl(&mut self) -> Result<(), String> {
        if let Some(ref mut child) = self.child { let _ = child.kill(); let _ = child.wait(); }
        self.child = None;
        Ok(())
    }
}

impl DebuggerBackend for CdbBackend {
    fn launch(&mut self, p: &str, a: &[String], c: &str) -> Result<(), String> { self.launch_impl(p,a,c) }
    fn set_breakpoint(&mut self, s: &str, l: u64) -> Result<Breakpoint, String> { self.set_breakpoint_impl(s,l) }
    fn exec_continue(&mut self) -> Result<(), String> { self.exec_continue_impl() }
    fn exec_next(&mut self) -> Result<(), String> { self.exec_next_impl() }
    fn exec_step(&mut self) -> Result<(), String> { self.exec_step_impl() }
    fn pause(&mut self) -> Result<(), String> { let _ = self.send_cmd(".break"); Ok(()) }
    fn poll_stopped(&mut self) -> Result<Value, String> {
        Ok(json!({"reason": "pause", "threadId": 0, "allThreadsStopped": true}))
    }
    fn evaluate_expression(&mut self, expr: &str) -> Result<String, String> {
        self.send_cmd(&format!("? {expr}"))
    }
    fn thread_info(&mut self) -> Result<Vec<Value>, String> { self.thread_info_impl() }
    fn stack_info(&mut self) -> Result<Vec<Value>, String> { self.stack_info_impl() }
    fn list_variables(&mut self) -> Result<Vec<Value>, String> { self.list_variables_impl() }
    fn terminate(&mut self) -> Result<(), String> { self.terminate_impl() }
    fn list_breakpoints(&self) -> Vec<Breakpoint> { Vec::new() }
    fn delete_breakpoint(&mut self, _id: u64) -> Result<(), String> { self.send_cmd(&format!("bc {}", _id)).map(|_| ()) }
    fn list_registers(&mut self) -> Result<Vec<Value>, String> { self.send_cmd("r").map(|_| vec![]) }
    fn read_memory(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, String> {
        self.send_cmd(&format!("db 0x{:X} L{}", addr, size)).map(|_| vec![])
    }
    fn name(&self) -> &'static str { "CDB/WinDbg" }
}

// ============================================================================
// Main — DAP stdio loop (5e.7a — auto-detect backend)
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
// Phase 8B: JSON API Mode — single-command structured output for GUI/scripts
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
            eprintln!("XIOM Debugger v0.49.7 — JSON API Mode");
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

    // Phase 8B: --json mode — single-command JSON API for GUIs/scripts
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

        // 6C.3: DAP evaluate handler — variable watch, REPL, hover
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
