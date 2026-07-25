// XIOM Debug Adapter Protocol Server — backends
// M14.1: Extracted from main.rs — GDB, CDB backends

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, Stdio};
use crate::{Breakpoint, DebuggerBackend};

pub(crate) struct GdbBackend {
    pub(crate) child: Option<Child>,
    pub(crate) reader: Option<BufReader<ChildStdout>>,
    pub(crate) breakpoints: HashMap<u64, Breakpoint>,
    pub(crate) next_breakpoint_id: u64,
    pub(crate) program_path: Option<String>,
}

impl GdbBackend {
    pub(crate) fn new() -> Self {
        GdbBackend { child: None, reader: None, breakpoints: HashMap::new(), next_breakpoint_id: 1, program_path: None }
    }

    fn launch_impl(&mut self, program: &str, args: &[String], _cwd: &str) -> Result<(), String> {
        if self.child.is_some() { return Err("Debug session already active".into()); }
        let mut cmd = Command::new("gdb");
        cmd.args(["--interpreter=mi3", "--quiet", program]);
        if !args.is_empty() { cmd.arg("--args"); for a in args { cmd.arg(a); } }
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());
        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn GDB: {e}"))?;
        let stdout = child.stdout.take().ok_or("Failed to capture GDB stdout")?;
        self.reader = Some(BufReader::new(stdout));
        self.child = Some(child);
        self.program_path = Some(program.to_string());
        Ok(())
    }

    pub(crate) fn send_mi(&mut self, cmd: &str) -> Result<String, String> {
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
            let trimmed = line.trim();
            if trimmed.starts_with('^') || trimmed.starts_with("*stopped") || trimmed.starts_with("*running") { break; }
        }
        Ok(response)
    }

    fn set_breakpoint_impl(&mut self, source: &str, line: u64) -> Result<Breakpoint, String> {
        let resp = self.send_mi(&format!("-break-insert --source \"{source}\" --line {line}"))?;
        let verified = resp.contains("^done");
        let bp = Breakpoint { id: self.next_breakpoint_id, source_path: source.to_string(), line, verified };
        self.breakpoints.insert(bp.id, bp.clone());
        self.next_breakpoint_id += 1;
        Ok(bp)
    }

    fn exec_continue_impl(&mut self) -> Result<(), String> { self.send_mi("-exec-continue").map(|_| ()) }
    fn exec_next_impl(&mut self) -> Result<(), String> { self.send_mi("-exec-next").map(|_| ()) }
    fn exec_step_impl(&mut self) -> Result<(), String> { self.send_mi("-exec-step").map(|_| ()) }

    fn thread_info_impl(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-thread-info")?;
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

    fn stack_info_impl(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-stack-list-frames")?;
        let mut frames = Vec::new();
        let mut frame_id = 0;
        for line in resp.lines() {
            if line.contains("frame={") {
                frame_id += 1;
                let mut name = "??".to_string(); let mut file = "unknown.xi".to_string(); let mut line_num: u64 = 0;
                if let Some(start) = line.find("func=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { name = rest[..end].to_string(); } }
                if let Some(start) = line.find("file=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { file = rest[..end].to_string(); } }
                if let Some(start) = line.find("line=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { line_num = rest[..end].parse().unwrap_or(0); } }
                frames.push(json!({"id": frame_id, "name": name, "source": {"name": file, "path": file}, "line": line_num, "column": 0}));
            }
        }
        Ok(frames)
    }

    fn list_variables_impl(&mut self) -> Result<Vec<Value>, String> {
        let resp = self.send_mi("-stack-list-variables --simple-values")?;
        let mut vars = Vec::new(); let mut var_ref = 1000;
        for line in resp.lines() {
            if line.contains("name=\"") {
                let mut name = String::new(); let mut value = String::new(); let mut var_type = "unknown".to_string();
                if let Some(start) = line.find("name=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { name = rest[..end].to_string(); } }
                if let Some(start) = line.find("value=\"") { let rest = &line[start + 7..]; if let Some(end) = rest.find('"') { value = rest[..end].to_string(); } }
                if let Some(start) = line.find("type=\"") { let rest = &line[start + 6..]; if let Some(end) = rest.find('"') { var_type = rest[..end].to_string(); } }
                if !name.is_empty() && name != "..." {
                    let is_compound = var_type.contains('*') || var_type.contains("struct") || var_type.contains("class");
                    let var_ref_id = if is_compound { var_ref += 1; var_ref } else { 0 };
                    vars.push(json!({"name": name, "value": value, "type": var_type, "variablesReference": var_ref_id}));
                }
            }
        }
        if vars.is_empty() { vars.push(json!({"name": "no locals", "value": "<no variables in scope>", "variablesReference": 0})); }
        Ok(vars)
    }

    fn terminate_impl(&mut self) -> Result<(), String> {
        if let Some(ref mut child) = self.child { let _ = child.kill(); let _ = child.wait(); }
        self.child = None;
        Ok(())
    }

    fn list_breakpoints_impl(&self) -> Vec<Breakpoint> { self.breakpoints.values().cloned().collect() }

    fn delete_breakpoint_impl(&mut self, id: u64) -> Result<(), String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let _ = self.send_mi(&format!("-break-delete {}", id));
        self.breakpoints.remove(&id);
        Ok(())
    }

    fn list_registers_impl(&mut self) -> Result<Vec<Value>, String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let resp = self.send_mi("-data-list-register-values x")?;
        let mut regs = Vec::new();
        if let Some(start) = resp.find("register-values=[") {
            let section = &resp[start + 17..];
            if let Some(end) = section.find(']') {
                for entry in section[..end].split("},") {
                    let val = entry.split("value=\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("?");
                    let num = entry.split("number=\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("?");
                    regs.push(json!({"name": format!("r{}", num), "value": val}));
                }
            }
        }
        Ok(regs)
    }

    fn read_memory_impl(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, String> {
        if self.child.is_none() { return Err("No debug session".into()); }
        let resp = self.send_mi(&format!("-data-read-memory-bytes 0x{:X} {}", addr, size))?;
        if let Some(start) = resp.find("contents=\"") {
            let bytes_str = &resp[start + 10..];
            if let Some(end) = bytes_str.find('"') {
                let hex = &bytes_str[..end];
                let mut bytes = Vec::new();
                for chunk in hex.split_whitespace() { if let Ok(b) = u8::from_str_radix(chunk, 16) { bytes.push(b); } }
                return Ok(bytes);
            }
        }
        Err("Cannot read memory".into())
    }
}

impl DebuggerBackend for GdbBackend {
    fn launch(&mut self, program: &str, args: &[String], cwd: &str) -> Result<(), String> { self.launch_impl(program, args, cwd) }
    fn set_breakpoint(&mut self, source: &str, line: u64) -> Result<Breakpoint, String> { self.set_breakpoint_impl(source, line) }
    fn exec_continue(&mut self) -> Result<(), String> { self.exec_continue_impl() }
    fn exec_next(&mut self) -> Result<(), String> { self.exec_next_impl() }
    fn exec_step(&mut self) -> Result<(), String> { self.exec_step_impl() }
    fn pause(&mut self) -> Result<(), String> {
        if let Some(ref child) = self.child {
            #[cfg(unix)] unsafe { libc::kill(child.id() as i32, libc::SIGINT); }
            #[cfg(windows)] { let _ = child; }
        }
        Ok(())
    }
    fn poll_stopped(&mut self) -> Result<Value, String> {
        if let Some(ref mut reader) = self.reader {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return Err("EOF".to_string()),
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with("*stopped") {
                        let reason = if trimmed.contains("breakpoint-hit") { "breakpoint" }
                            else if trimmed.contains("end-stepping-range") { "step" } else { "pause" };
                        let desc = trimmed.replacen("*stopped,", "", 1);
                        return Ok(json!({"reason": reason, "description": desc, "threadId": 1, "allThreadsStopped": true}));
                    } else if trimmed.starts_with("*running") { return Ok(json!({"reason": "continued", "threadId": 1})); }
                    return Ok(json!({"reason": "unknown"}));
                }
                Err(e) => return Err(format!("read error: {e}")),
            }
        }
        Err("no reader".to_string())
    }
    fn evaluate_expression(&mut self, expr: &str) -> Result<String, String> {
        let resp = self.send_mi(&format!("-data-evaluate-expression \"{expr}\""))?;
        if let Some(val_start) = resp.find("value=\"") {
            let after = &resp[val_start + 7..];
            if let Some(val_end) = after.find('"') { return Ok(after[..val_end].to_string()); }
        }
        Ok(resp)
    }
    fn thread_info(&mut self) -> Result<Vec<Value>, String> { self.thread_info_impl() }
    fn stack_info(&mut self) -> Result<Vec<Value>, String> { self.stack_info_impl() }
    fn list_variables(&mut self) -> Result<Vec<Value>, String> { self.list_variables_impl() }
    fn terminate(&mut self) -> Result<(), String> { self.terminate_impl() }
    fn list_breakpoints(&self) -> Vec<Breakpoint> { self.list_breakpoints_impl() }
    fn delete_breakpoint(&mut self, id: u64) -> Result<(), String> { self.delete_breakpoint_impl(id) }
    fn list_registers(&mut self) -> Result<Vec<Value>, String> { self.list_registers_impl() }
    fn read_memory(&mut self, a: u64, s: usize) -> Result<Vec<u8>, String> { self.read_memory_impl(a, s) }
    fn name(&self) -> &'static str { "GDB/MI" }
}

// CDB/WinDbg Backend
pub(crate) struct CdbBackend {
    pub(crate) child: Option<Child>,
    pub(crate) breakpoints: HashMap<u64, Breakpoint>,
    pub(crate) next_breakpoint_id: u64,
    pub(crate) program_path: Option<String>,
}

impl CdbBackend {
    pub(crate) fn new() -> Self {
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
    fn evaluate_expression(&mut self, expr: &str) -> Result<String, String> { self.send_cmd(&format!("? {expr}")) }
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
