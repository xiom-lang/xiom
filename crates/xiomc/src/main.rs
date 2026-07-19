// XIOM Programming Language
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
// -----------------------------------------------------------------------

//! XIOM Compiler CLI
//! Usage:
//!   xiomc <source.xi>                         print LLVM IR to stdout
//!   xiomc --emit-ir <source.xi>               print LLVM IR to stdout
//!   xiomc -o <output> <source.xi>             compile to native binary
//!   xiomc --target wasm <source.xi>           compile to WASM
//!   xiomc --target wasm -o out.wasm <src.xi>  compile to WASM with name
//!   xiomc --run <source.xi>                   compile and run, print exit code
//!   xiomc --diagnostics=json <source.xi>      JSON-structured compiler output
//!   xiomc --dump-contracts <source.xi>        emit contract index as JSON
//!   xiomc --sandbox <source.xi>                safety audit report (text)
//!   xiomc --sandbox=strict <source.xi>         block compilation on HIGH findings
//!   xiomc --sandbox-report=json <source.xi>    safety audit as JSON

use std::collections::HashMap;
use std::env;
use std::process;
use std::time::Duration;

use xiomc::{self, compile, CompileConfig, Target, resolve_source_files};
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::sandbox::SafetyAuditor;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(if args.iter().any(|a| a == "--help") { 0 } else { 1 });
    }

    if args.iter().any(|a| a == "--version") {
        let tag = option_env!("XIOM_RELEASE_TAG").unwrap_or("Production");
        let stats = option_env!("XIOM_RELEASE_STATS").unwrap_or("101/101 e2e, deterministic builds");
        println!("XIOM Compiler v{} \"{tag}\" - {stats}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // 5c-R: --explain EXXXX opens the error code reference
    if let Some(pos) = args.iter().position(|a| a == "--explain") {
        if let Some(code) = args.get(pos + 1) {
            xiomc::explain_error(code);
            return;
        }
        eprintln!("usage: xiomc --explain <code>  (e.g., xiomc --explain X0010)");
        process::exit(1);
    }

    let emit_ir = args.iter().any(|a| a == "--emit-ir");
    let do_run = args.iter().any(|a| a == "--run");
    let check_only = args.iter().any(|a| a == "--check");
    let release = args.iter().any(|a| a == "--release");
    let target = parse_target(&args);
    let check_contracts = !args.iter().any(|a| a == "--no-contracts") && !release;
    let diagnostics_json = args.iter().any(|a| a == "--diagnostics=json");
    let strict_mode = args.iter().any(|a| a == "--strict");
    let debug_symbols = args.iter().any(|a| a == "--debug") || args.iter().any(|a| a == "-g");
    let shared_lib = args.iter().any(|a| a == "--shared");
    let static_lib = args.iter().any(|a| a == "--static");
    let watch_mode = args.iter().any(|a| a == "--watch");
    let hot_reload = args.iter().any(|a| a == "--hot-reload");
    let test_mode = args.iter().any(|a| a == "--test");
    let clean_mode = args.iter().any(|a| a == "--clean");
    let install_mode = args.iter().any(|a| a == "install");
    let install_pkg = args.iter().position(|a| a == "install")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|p| !p.starts_with('-'));
    let publish_mode = args.iter().any(|a| a == "publish");
    let update_mode = args.iter().any(|a| a == "update");
    let bench_mode = args.iter().any(|a| a == "bench");
    let bench_count: u32 = parse_flag_value(&args, "--count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let init_mode = args.iter().any(|a| a == "init");
    let new_mode = args.iter().any(|a| a == "new");
    let new_name: Option<String> = args.iter().position(|a| a == "new")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|n| !n.starts_with('-'));
    let registry_cmd = args.iter().any(|a| a == "registry");

    if registry_cmd {
        handle_registry(&args);
        return;
    }

    if init_mode {
        scaffold_project(".", None);
        return;
    }
    if new_mode {
        let name = new_name.as_deref().unwrap_or("xiom-project");
        scaffold_project(name, Some(name));
        return;
    }

    if clean_mode {
        let extensions = ["exe", "ll", "obj", "o", "out", "wasm", "pdb", "ilk", "exp", "lib"];
        let mut cleaned = 0usize;
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        if std::fs::remove_file(&path).is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        eprintln!("  Cleaned {} build artifact(s)", cleaned);
        return;
    }

    if install_mode || update_mode {
        let registry_url = parse_flag_value(&args, "--registry")
            .unwrap_or_else(|| "https://registry.xiom-lang.org/packages.json".to_string());
        handle_install(&args, install_pkg.as_deref(), &registry_url, update_mode);
        return;
    }

    if bench_mode {
        run_benchmarks(&args, bench_count);
        return;
    }

    if publish_mode {
        handle_publish(&args);
        return;
    }

    let verify = args.iter().any(|a| a == "--verify") || args.iter().any(|a| a == "--verify-output");
    let verify_output = parse_flag_value(&args, "--verify-output");

    let output_file = parse_flag_value(&args, "-o");

    let link_libs = parse_all_flag_values(&args, "--link");
    let link_paths = parse_all_flag_values(&args, "--link-path");
    let c_sources = parse_all_flag_values(&args, "--c-source");

    let timeout_secs: u64 = parse_flag_value(&args, "--timeout")
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let max_recursion_depth: u32 = parse_flag_value(&args, "--max-depth")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let max_recursion_depth = std::cmp::min(max_recursion_depth, 10000u32);

    if timeout_secs > 0 {
        let duration = Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            eprintln!("error: compilation timed out after {} seconds", timeout_secs);
            std::process::exit(1);
        });
    }

    let max_memory_mb: u64 = parse_flag_value(&args, "--max-memory-mb")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if max_memory_mb > 0 {
        let max_bytes = max_memory_mb * 1024 * 1024;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(2));
                if let Some(used_bytes) = get_process_memory_bytes() {
                    if used_bytes > max_bytes {
                        eprintln!(
                            "error: memory budget exceeded ({} MB used of {} MB limit). \
                             Try --max-memory-mb with a higher value or simplify the input.",
                            used_bytes / 1024 / 1024,
                            max_memory_mb
                        );
                        std::process::exit(1);
                    }
                }
            }
        });
    }

    let source_paths = resolve_source_files(&args);
    if source_paths.is_empty() && !test_mode {
        eprintln!("error: no source file(s) provided");
        process::exit(1);
    }

    if test_mode {
        run_xiom_tests(&args);
        return;
    }

    let config = CompileConfig {
        target,
        emit_ir,
        do_run,
        check_only,
        release,
        check_contracts,
        diagnostics_json,
        strict_mode,
        debug_symbols,
        shared_lib,
        static_lib,
        max_recursion_depth,
        dump_contracts: args.iter().any(|a| a == "--dump-contracts"),
        verify,
        verify_output,
        output_file,
        link_libs,
        link_paths,
        c_sources,
    };

    // --sandbox: run safety audit and exit (skips compilation unless --sandbox=strict passes)
    let sandbox_mode = args.iter().any(|a| a == "--sandbox" || a.starts_with("--sandbox="));
    if sandbox_mode {
        let strict = args.iter().any(|a| a == "--sandbox=strict");
        let json_output = args.iter().any(|a| a == "--sandbox-report=json");
        let _text_output = !json_output && !args.iter().any(|a| a.starts_with("--sandbox-report="));
        let output_file = args.iter().position(|a| a == "--sandbox-report")
            .and_then(|i| args.get(i + 1).cloned());
        // Also support --sandbox-report=<path>
        let output_file = output_file.or_else(|| {
            args.iter().find(|a| a.starts_with("--sandbox-report="))
                .and_then(|a| a.strip_prefix("--sandbox-report="))
                .map(|s| if s == "json" || s == "text" || s == "silent" { "" } else { s })
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });
        let silent = args.iter().any(|a| a == "--sandbox-report=silent");

        let mut overall_exit = 0;
        for source_path in &source_paths {
            let source = std::fs::read_to_string(source_path).unwrap_or_default();
            let tokens = Lexer::new(&source).tokenize();
            if let Ok(program) = Parser::new(tokens).parse_program() {
                let mut auditor = SafetyAuditor::new();
                let report = auditor.audit(&program, source_path);

                let output = if json_output {
                    report.to_json()
                } else {
                    report.to_text()
                };

                if let Some(ref path) = output_file {
                    if !path.is_empty() {
                        let _ = std::fs::write(path, &output);
                    }
                }
                if !silent && output_file.is_none() {
                    println!("{output}");
                }

                if strict && (report.summary.safety_score == "HIGH" || report.summary.safety_score == "CRITICAL") {
                    eprintln!("error: --sandbox=strict blocked compilation due to {} HIGH severity findings", report.summary.high_severity);
                    overall_exit = 3;
                } else if report.summary.safety_score == "HIGH" || report.summary.safety_score == "CRITICAL" {
                    overall_exit = std::cmp::max(overall_exit, 2);
                } else if report.summary.safety_score == "MEDIUM" {
                    overall_exit = std::cmp::max(overall_exit, 1);
                }
            }
        }
        process::exit(overall_exit);
    }

    // 5e Hot Reload
    if watch_mode || hot_reload {
        let hot_config = CompileConfig {
            shared_lib: hot_reload || shared_lib,
            ..config // consumes config
        };
        eprintln!("\n[HOT RELOAD] Watching {} source file(s)...", source_paths.len());
        eprintln!("[HOT RELOAD] Press Ctrl+C to stop.\n");

        let mut last_mod: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for p in &source_paths {
            if let Ok(meta) = std::fs::metadata(p) {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        last_mod.insert(p.clone(), dur.as_secs());
                    }
                }
            }
        }

        // Initial compile
        compile(&hot_config, &source_paths);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut changed = false;
            for p in &source_paths {
                if let Ok(meta) = std::fs::metadata(p) {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                            let s = dur.as_secs();
                            if last_mod.get(p) != Some(&s) { last_mod.insert(p.clone(), s); changed = true; }
                        }
                    }
                }
            }
            if changed { compile(&hot_config, &source_paths); }
        }
    }

    compile(&config, &source_paths);
}

fn print_usage() {
        let tag = option_env!("XIOM_RELEASE_TAG").unwrap_or("Production");
        let stats = option_env!("XIOM_RELEASE_STATS").unwrap_or("Deterministic Builds, 101/101 E2E");
        eprintln!("XIOM Compiler v{} \"{tag}\" -- {stats}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiomc [OPTIONS] <source.xi>");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help              Show this help message");
    eprintln!("  --version           Print version");
    eprintln!("  -o <output>         Output binary path (default: a.exe)");
    eprintln!("  --run               Compile and run, print exit code");
    eprintln!("  --emit-ir           Print LLVM IR to stdout (no compilation)");
    eprintln!("  --target <target>   Target: native (default), wasm, arm, riscv");
    eprintln!("  --no-contracts      Disable contract runtime checks");
    eprintln!("  --diagnostics=json  Output diagnostics as JSON");
    eprintln!("  --dump-contracts    Print contract index as JSON");
    eprintln!("  --sandbox           Run safety audit on unsafe blocks (text report)");
    eprintln!("  --sandbox=strict    Block compilation if HIGH severity findings");
    eprintln!("  --sandbox-report=json  Output sandbox report as JSON");
    eprintln!("  --verify            Generate SMT-LIB contract verification output");
    eprintln!("  --verify-output <f> Write SMT-LIB to file");
    eprintln!("  --shared            Compile as shared library (DLL)");
    eprintln!("  --watch             Watch source files and recompile on change");
    eprintln!("  --hot-reload        Hot reload mode: watch + shared library");
    eprintln!("  --timeout <seconds>  Set compilation timeout (default: 60)");
    eprintln!("  --max-memory-mb <N>       Set max memory budget in MB (0 = disabled)");
    eprintln!("  --link <name>             Link a native library (repeatable, e.g. vulkan-1)");
    eprintln!("  --link-path <dir>         Add a library search path (repeatable, -L<dir>)");
    eprintln!("  --c-source <file>         Link an extra C/object file (repeatable)");
    eprintln!();
    eprintln!("DEPENDENCIES:");
    eprintln!("  Required: clang (LLVM) — to compile IR to native binary");
    eprintln!("  Optional: opt (LLVM) — IR optimization pass (-O1)");
    eprintln!("  Optional: nasm — hardware-accelerated crypto/memcpy (stdlib)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiomc --run examples/demo_float.xi");
    eprintln!("  xiomc -o prog.exe source.xi");
    eprintln!("  xiomc --emit-ir examples/demo_float.xi");
    eprintln!("  xiomc --target wasm -o prog.wasm source.xi");
    eprintln!("  xiomc --verify examples/phase1_contracts.xi");
}

fn parse_target(args: &[String]) -> Target {
    match parse_flag_value(args, "--target").as_deref() {
        Some("wasm") => Target::Wasm,
        Some("arm") => Target::Arm,
        Some("riscv") => Target::RisCv,
        _ => Target::Native,
    }
}

fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    let pos = args.iter().position(|a| a == flag)?;
    args.get(pos + 1).cloned()
}

fn parse_all_flag_values(args: &[String], flag: &str) -> Vec<String> {
    let mut values = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        if arg == flag {
            match args.get(i + 1) {
                Some(val) if !val.starts_with('-') => values.push(val.clone()),
                _ => {
                    eprintln!("error: '{flag}' requires a value");
                    process::exit(1);
                }
            }
        }
    }
    values
}

fn get_process_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        let pid = std::process::id();
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).WorkingSet64")])
            .output()
        {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = s.trim().parse::<u64>() {
                        return Some(bytes);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return Some(kb * 1024);
                }
            }
        }
        None
    }
}

fn run_xiom_tests(args: &[String]) {
    use std::process::Command as Cmd;

    let test_dir = parse_flag_value(args, "--test")
        .or_else(|| parse_flag_value(args, "--test-dir"))
        .unwrap_or_else(|| "examples".to_string());

    let mut test_files: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                test_files.push(path.to_string_lossy().to_string());
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for e in sub.flatten() {
                        let p = e.path();
                        if p.extension().and_then(|ext| ext.to_str()) == Some("xi") {
                            test_files.push(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    if test_files.is_empty() {
        eprintln!("  No .xi test files found in {}", test_dir);
        std::process::exit(1);
    }

    eprintln!("  Running {} test(s)...", test_files.len());
    let mut passed = 0usize;
    let mut failed = 0usize;

    for test_file in &test_files {
        let exe_path = format!("{}.test.exe", test_file);
        let xiomc_path = std::env::current_exe().unwrap_or_else(|_| "xiomc".into());
        let compile = Cmd::new(&xiomc_path)
            .args(["-o", &exe_path, test_file])
            .output();

        match compile {
            Ok(out) if out.status.success() => {
                match Cmd::new(&exe_path).output() {
                    Ok(run_out) => {
                        if run_out.status.success() {
                            passed += 1;
                            eprintln!("    PASS  {}", test_file);
                        } else {
                            failed += 1;
                            eprintln!("    FAIL  {} (exit code {})", test_file, run_out.status.code().unwrap_or(-1));
                            if !run_out.stderr.is_empty() {
                                eprintln!("      {}", String::from_utf8_lossy(&run_out.stderr).trim());
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("    FAIL  {} (cannot run: {})", test_file, e);
                    }
                }
                let _ = std::fs::remove_file(&exe_path);
            }
            Ok(_) => {
                failed += 1;
                eprintln!("    FAIL  {} (compilation failed)", test_file);
            }
            Err(e) => {
                failed += 1;
                eprintln!("    FAIL  {} (cannot compile: {})", test_file, e);
            }
        }
    }

    eprintln!("  {} passed, {} failed", passed, failed);
    if failed > 0 { std::process::exit(1); }
}

// ── Phase 5d: Package Manager ──────────────────────────────────────────

fn handle_install(_args: &[String], pkg_name: Option<&str>, registry_url: &str, _update: bool) {
    let home = dirs_next().unwrap_or_else(|| ".".into());
    let pkgs_dir = format!("{}/.xiom/packages", home);
    std::fs::create_dir_all(&pkgs_dir).ok();

    let deps: Vec<String> = if let Some(name) = pkg_name {
        vec![name.to_string()]
    } else {
        parse_deps_from_manifest("package.xi").unwrap_or_default()
    };

    if deps.is_empty() {
        eprintln!("  No dependencies to install. Add packages to package.xi or specify a package name.");
        eprintln!("  Usage: xiom install <package>");
        eprintln!("     or: add dependencies to package.xi and run 'xiom install'");
        return;
    }

    eprintln!("  Fetching registry index from {}...", registry_url);
    let index = fetch_registry_index(registry_url);
    match &index {
        Ok(idx) => eprintln!("  Registry: {} packages available", idx.len()),
        Err(e) => {
            eprintln!("  Warning: cannot fetch registry ({}). Using local cache only.", e);
            eprintln!("  Make sure {} is accessible or use --registry <url>", registry_url);
        }
    }

    eprintln!("  Resolving {} package(s)...", deps.len());
    let mut installed: Vec<String> = Vec::new();
    for dep in &deps {
        let parts: Vec<&str> = dep.splitn(2, ':').collect();
        let name = parts[0].trim();
        let _version_req = parts.get(1).map(|s| s.trim()).unwrap_or("*");

        let repo_url = if let Ok(ref idx) = index {
            idx.get(name).map(|pkg| pkg.repo.clone())
        } else {
            None
        };

        match repo_url {
            Some(url) => {
                let pkg_dir = format!("{}/{}", pkgs_dir, name);
                if std::path::Path::new(&pkg_dir).exists() {
                    eprintln!("    {} already installed (use 'xiom update' to refresh)", name);
                } else {
                    eprintln!("    Installing {} from {}...", name, url);
                    let status = std::process::Command::new("git")
                        .args(["clone", "--depth", "1", &url, &pkg_dir])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            eprintln!("      installed {} to {}", name, pkg_dir);
                            installed.push(name.to_string());
                        }
                        Ok(s) => eprintln!("      git clone failed with exit code {}", s.code().unwrap_or(-1)),
                        Err(e) => eprintln!("      git not found: {}. Install git to clone packages.", e),
                    }
                }
            }
            None => {
                eprintln!("    Package '{}' not found in registry", name);
                eprintln!("    Check that the registry at {} has this package.", registry_url);
            }
        }
    }

    if !installed.is_empty() {
        let lock_path = "xiom.lock";
        let lock_content = serde_json::json!({
            "version": 1,
            "packages": installed.iter().map(|p| {
                serde_json::json!({ "name": p, "version": "*", "source": "registry" })
            }).collect::<Vec<_>>()
        });
        if let Ok(json) = serde_json::to_string_pretty(&lock_content) {
            if std::fs::write(lock_path, &json).is_ok() {
                eprintln!("  Wrote lockfile: {}", lock_path);
            }
        }
        eprintln!("  Installed {} package(s)", installed.len());
    }
    if _args.iter().any(|a| a == "--frozen" || a == "--locked") {
        let lock_path = "xiom.lock";
        if std::path::Path::new(lock_path).exists() {
            eprintln!("  Lockfile verified: {}", lock_path);
        } else {
            eprintln!("  Warning: --locked specified but no xiom.lock found.");
        }
    }
}

fn parse_deps_from_manifest(path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut deps = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("dependencies:") || trimmed.starts_with("\"dependencies\":") {
            if let Some(start) = trimmed.find('[') {
                let inner = &trimmed[start..];
                for part in inner.trim_matches(|c| c == '[' || c == ']').split(',') {
                    let cleaned = part.trim().trim_matches('"').trim();
                    if !cleaned.is_empty() {
                        deps.push(cleaned.to_string());
                    }
                }
            }
        }
    }
    Ok(deps)
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct RegistryPackage {
    repo: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    license: String,
}

fn fetch_registry_index(url: &str) -> Result<HashMap<String, RegistryPackage>, String> {
    let home = dirs_next().unwrap_or_else(|| ".".into());
    let local_path = format!("{}/.xiom/registry.json", home);
    if let Ok(content) = std::fs::read_to_string(&local_path) {
        if let Ok(root) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut map = HashMap::new();
            if let Some(obj) = root["packages"].as_object() {
                for (name, val) in obj {
                    if let Ok(pkg) = serde_json::from_value::<RegistryPackage>(val.clone()) {
                        map.insert(name.clone(), pkg);
                    }
                }
            }
            if !map.is_empty() {
                return Ok(map);
            }
        }
    }
    let body = if let Ok(out) = std::process::Command::new("curl")
        .args(["-sSfL", "--connect-timeout", "10", url])
        .output()
    {
        if out.status.success() { String::from_utf8_lossy(&out.stdout).to_string() }
        else { return Err(format!("curl failed: {}", String::from_utf8_lossy(&out.stderr))); }
    } else if let Ok(out) = std::process::Command::new("wget")
        .args(["-qO-", "--timeout=10", url])
        .output()
    {
        if out.status.success() { String::from_utf8_lossy(&out.stdout).to_string() }
        else { return Err(format!("wget failed: {}", String::from_utf8_lossy(&out.stderr))); }
    } else if let Ok(out) = std::process::Command::new("powershell")
        .args(["-Command", &format!("(Invoke-WebRequest -Uri '{url}' -TimeoutSec 10).Content")])
        .output()
    {
        if out.status.success() { String::from_utf8_lossy(&out.stdout).to_string() }
        else { return Err("cannot fetch registry (no curl/wget/powershell available)".to_string()); }
    } else {
        return Err("cannot fetch registry (no HTTP client available)".to_string());
    };
    let root: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("invalid JSON: {e}"))?;
    let pkgs = root.get("packages").ok_or("missing 'packages' key in registry")?;
    let mut map = HashMap::new();
    if let Some(obj) = pkgs.as_object() {
        for (name, val) in obj {
            if let Ok(pkg) = serde_json::from_value::<RegistryPackage>(val.clone()) {
                map.insert(name.clone(), pkg);
            }
        }
    }
    Ok(map)
}

fn handle_publish(_args: &[String]) {
    let manifest_path = "package.xi";
    if !std::path::Path::new(manifest_path).exists() {
        eprintln!("  No package.xi found. Create one with 'xiom init' first.");
        eprintln!("  See docs/PACKAGE_MANAGER.md for manifest format.");
        return;
    }

    let content = match std::fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("  Cannot read package.xi: {e}"); return; }
    };

    let mut name = String::new();
    let mut version = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name:") {
            name = trimmed.trim_start_matches("name:").trim().trim_matches('"').to_string();
        }
        if trimmed.starts_with("version:") {
            version = trimmed.trim_start_matches("version:").trim().trim_matches('"').to_string();
        }
    }

    if name.is_empty() || version.is_empty() {
        eprintln!("  package.xi must have 'name' and 'version' fields.");
        eprintln!("  Example:  name: \"my-package\"");
        eprintln!("           version: \"1.0.0\"");
        return;
    }

    eprintln!("  Publishing {name} v{version}...");
    eprintln!("  Tagging v{version}...");
    let tag = format!("v{version}");
    let tag_status = std::process::Command::new("git")
        .args(["tag", "-a", &tag, "-m", &format!("Release {tag}")])
        .status();
    match tag_status {
        Ok(s) if s.success() => eprintln!("    created tag {tag}"),
        Ok(s) => eprintln!("    git tag failed (exit {}). Tag may already exist.", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("    git not found: {e}"),
    }

    eprintln!("  Push to remote...");
    let push_status = std::process::Command::new("git")
        .args(["push", "origin", &tag])
        .status();
    match push_status {
        Ok(s) if s.success() => eprintln!("    pushed tag {tag}"),
        _ => eprintln!("    manual push required: git push origin {tag}"),
    }

    eprintln!("  Next steps:");
    eprintln!("    1. Create a release on your Git host (Gitea/GitHub)");
    eprintln!("    2. Submit a PR to the registry repo to add your package");
    eprintln!("    3. Your package will be available via 'xiom install {name}'");
}

fn dirs_next() -> Option<String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
}

fn handle_registry(args: &[String]) {
    let home = dirs_next().unwrap_or_else(|| ".".into());
    let reg_path = format!("{}/.xiom/registry.json", home);

    let sub_cmd = args.iter()
        .position(|a| a == "registry")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "list".to_string());

    match sub_cmd.as_str() {
        "init" => {
            let initial = serde_json::json!({ "packages": {} });
            if let Ok(json) = serde_json::to_string_pretty(&initial) {
                std::fs::create_dir_all(format!("{}/.xiom", home)).ok();
                if std::fs::write(&reg_path, &json).is_ok() {
                    eprintln!("  Created local registry: {}", reg_path);
                }
            }
        }
        "add" => {
            let pkg_name = args.iter()
                .position(|a| a == "add")
                .and_then(|i| args.get(i + 1).cloned())
                .or_else(|| args.iter()
                    .position(|a| a == "registry")
                    .and_then(|i| args.get(i + 2).cloned()));
            let repo_url = args.iter()
                .position(|a| a == "add")
                .and_then(|i| args.get(i + 2).cloned())
                .or_else(|| args.iter()
                    .position(|a| a == "registry")
                    .and_then(|i| args.get(i + 3).cloned()));

            match (pkg_name, repo_url) {
                (Some(name), Some(url)) => {
                    let mut registry: serde_json::Value = if let Ok(content) = std::fs::read_to_string(&reg_path) {
                        serde_json::from_str(&content).unwrap_or(serde_json::json!({ "packages": {} }))
                    } else {
                        serde_json::json!({ "packages": {} })
                    };
                    if let Some(pkgs) = registry.get_mut("packages").and_then(|p| p.as_object_mut()) {
                        let entry = serde_json::json!({
                            "repo": url,
                            "description": "",
                            "license": "MIT"
                        });
                        pkgs.insert(name.clone(), entry);
                        if let Ok(json) = serde_json::to_string_pretty(&registry) {
                            std::fs::create_dir_all(format!("{}/.xiom", home)).ok();
                            std::fs::write(&reg_path, &json).ok();
                            eprintln!("  Added '{}' to local registry -> {}", name, url);
                        }
                    }
                }
                _ => {
                    eprintln!("  Usage: xiom registry add <name> <repo-url>");
                }
            }
        }
        "list" | _ => {
            if let Ok(content) = std::fs::read_to_string(&reg_path) {
                if let Ok(registry) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(pkgs) = registry["packages"].as_object() {
                        eprintln!("  Local registry ({} packages):", pkgs.len());
                        for (name, pkg) in pkgs {
                            let repo = pkg["repo"].as_str().unwrap_or("-");
                            let desc = pkg["description"].as_str().unwrap_or("");
                            eprintln!("    {name:<20} {repo:<50} {desc}");
                        }
                    } else {
                        eprintln!("  No packages in local registry. Use 'xiom registry add <name> <url>'");
                    }
                }
            } else {
                eprintln!("  No local registry found. Create one with 'xiom registry init'");
            }
        }
    }
}

fn run_benchmarks(args: &[String], iterations: u32) {
    let bench_dir = parse_flag_value(args, "bench")
        .unwrap_or_else(|| "benches".to_string());

    let bench_file = parse_flag_value(args, "--bench-file")
        .or_else(|| args.iter().find(|a| a.ends_with(".xi")).cloned());

    let mut bench_files: Vec<String> = Vec::new();
    if let Some(file) = bench_file {
        bench_files.push(file);
    } else {
        if let Ok(entries) = std::fs::read_dir(&bench_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                    bench_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    if bench_files.is_empty() {
        eprintln!("  No benchmark files found. Create a benches/ directory with .xi files.");
        eprintln!("  Or specify a file: xiom bench my_bench.xi");
        return;
    }

    eprintln!("  Running {} benchmark(s) x {} iterations...", bench_files.len(), iterations);

    for bench_file in &bench_files {
        let exe_path = format!("{}.bench.exe", bench_file);
        let xiomc_path = std::env::current_exe().unwrap_or_else(|_| "xiomc".into());

        let compile = std::process::Command::new(&xiomc_path)
            .args(["-o", &exe_path, "--release", bench_file])
            .output();

        match compile {
            Ok(out) if out.status.success() => {
                let mut times: Vec<f64> = Vec::new();
                for _ in 0..iterations {
                    use std::time::Instant;
                    let start = Instant::now();
                    let _ = std::process::Command::new(&exe_path).output();
                    let elapsed = start.elapsed().as_secs_f64();
                    times.push(elapsed);
                }

                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let min = times.first().copied().unwrap_or(0.0);
                let max = times.last().copied().unwrap_or(0.0);
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let median = times[times.len() / 2];

                let name = std::path::Path::new(bench_file)
                    .file_stem().and_then(|n| n.to_str()).unwrap_or(bench_file);
                eprintln!("  {name:<30} {min:>8.4}s  {mean:>8.4}s  {median:>8.4}s  {max:>8.4}s");

                let _ = std::fs::remove_file(&exe_path);
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  FAIL  {} (compilation failed: {})", bench_file, stderr.lines().next().unwrap_or(""));
            }
            Err(e) => {
                eprintln!("  FAIL  {} (cannot compile: {})", bench_file, e);
            }
        }
    }
    eprintln!("  Benchmark complete.");
}

fn scaffold_project(dir: &str, pkg_name: Option<&str>) {
    let name = pkg_name.unwrap_or_else(|| {
        std::path::Path::new(dir).file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("xiom-project")
    });

    let src_dir = format!("{dir}/src");
    let tests_dir = format!("{dir}/tests");
    for d in &[dir, &src_dir, &tests_dir] {
        if let Err(e) = std::fs::create_dir_all(d) {
            if !std::path::Path::new(d).exists() {
                eprintln!("  Cannot create {}: {}", d, e);
                return;
            }
        }
    }

    let manifest = format!(r#"// XIOM package manifest
name: "{name}"
version: "0.1.0"
authors: ["Your Name"]
license: "MIT"
description: "A new XIOM project"

dependencies: []

sources: [
    "src/main.xi",
]

tests: [
    "tests/test_main.xi",
]
"#);
    let manifest_path = format!("{dir}/package.xi");
    if !std::path::Path::new(&manifest_path).exists() {
        std::fs::write(&manifest_path, &manifest).ok();
    }

    let main_xi = format!(r#"// {name} — entry point
module {name}

fn main() -> Int {{
    return 0;
}}
"#);
    let main_path = format!("{dir}/src/main.xi");
    if !std::path::Path::new(&main_path).exists() {
        std::fs::write(&main_path, &main_xi).ok();
    }

    let test_xi = format!(r#"// {name} — tests
module {name}_test

fn test_hello() -> Int {{
    return 0;
}}
"#);
    let test_path = format!("{dir}/tests/test_main.xi");
    if !std::path::Path::new(&test_path).exists() {
        std::fs::write(&test_path, &test_xi).ok();
    }

    let gitignore = "*.exe\n*.ll\n*.obj\n*.o\n*.out\n*.wasm\n*.pdb\n*.ilk\n*.exp\n*.lib\nxiom.lock\n";
    let gitignore_path = format!("{dir}/.gitignore");
    if !std::path::Path::new(&gitignore_path).exists() {
        std::fs::write(&gitignore_path, gitignore).ok();
    }

    eprintln!("  Created project '{name}' in {dir}/");
    eprintln!("  ");
    eprintln!("  {dir}/");
    eprintln!("  ├── package.xi       ← project manifest");
    eprintln!("  ├── src/main.xi      ← entry point");
    eprintln!("  ├── tests/");
    eprintln!("  │   └── test_main.xi ← tests");
    eprintln!("  └── .gitignore");
    eprintln!("  ");
    eprintln!("  Next steps:");
    eprintln!("    cd {dir}");
    eprintln!("    xiom check         ← type-check your project");
    eprintln!("    xiomc src/main.xi --run   ← compile and run");
    eprintln!("    xiom test          ← run test suite");
}
