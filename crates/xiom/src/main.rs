// XIOM Programming Language
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
// -----------------------------------------------------------------------

//! XIOM Compiler CLI
//! Usage:
//!   xiom <source.xi>                         print LLVM IR to stdout
//!   xiom --emit-ir <source.xi>               print LLVM IR to stdout
//!   xiom -o <output> <source.xi>             compile to native binary
//!   xiom --target wasm <source.xi>           compile to WASM
//!   xiom --target wasm -o out.wasm <src.xi>  compile to WASM with name
//!   xiom --run <source.xi>                   compile and run, print exit code
//!   xiom --diagnostics=json <source.xi>      JSON-structured compiler output
//!   xiom --dump-contracts <source.xi>        emit contract index as JSON
//!   xiom --sandbox <source.xi>                safety audit report (text)
//!   xiom --sandbox=strict <source.xi>         block compilation on HIGH findings
//!   xiom --sandbox-report=json <source.xi>    safety audit as JSON

use std::collections::HashMap;
use std::env;
use std::process;
use std::time::Duration;

use xiom::{self, compile, CompileConfig, Target, resolve_source_files};

mod cli;

/// Call compile() and exit on failure -- all process::exit calls are confined to this binary.
fn compile_or_exit(config: &CompileConfig, sources: &[String]) {
    if let Err(errors) = compile(config, sources) {
        for e in &errors {
            eprintln!("error: {e}");
        }
        process::exit(1);
    }
}

/// M10: Watch mode for `xiom run --watch <file>`.
/// Polls the source file every 500ms and re-runs on changes.
fn run_script_watch(path: &str) {
    let get_mtime = || std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
    let mut last_mtime = get_mtime();

    eprintln!("[WATCH] Monitoring '{path}' -- press Ctrl+C to stop");
    // Unique per watch session: parallel watchers must not collide on a
    // shared `_script_watch.xi` in the temp dir.
    let watch_suffix = xiom::jit::rand_suffix();

    loop {
        let current_mtime = get_mtime();
        if current_mtime != last_mtime {
            if current_mtime.is_some() {
                // Debounce: wait 200ms for the file write to complete
                std::thread::sleep(std::time::Duration::from_millis(200));
                eprintln!("\n[WATCH] File changed, re-running...");
                let source = match std::fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(e) => { eprintln!("error: cannot read '{path}': {e}"); process::exit(1); }
                };
                let wrapped = xiom::implicit_main::wrap_implicit_main(&source);
                let tmp_dir = std::env::temp_dir().join("xiom_run");
                let _ = std::fs::create_dir_all(&tmp_dir);
                let tmp_src = tmp_dir.join(format!("_script_watch_{watch_suffix:x}.xi"));
                let tmp_out = tmp_dir.join(format!("_script_watch_{watch_suffix:x}.exe"));
                std::fs::write(&tmp_src, &wrapped).unwrap_or_else(|e| {
                    eprintln!("error: write temp: {e}"); process::exit(1);
                });
                let config = CompileConfig {
                    output_file: Some(tmp_out.to_string_lossy().to_string()),
                    do_run: true,
                    ..CompileConfig::default()
                };
                if let Err(errors) = compile(&config, &[tmp_src.to_string_lossy().to_string()]) {
                    for e in &errors { eprintln!("error: {e}"); }
                }
            }
            last_mtime = current_mtime;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::sandbox::SafetyAuditor;

/// M10.4: Interactive REPL -- compile and execute each line as a script.
/// State (let/var declarations) persists across lines.
fn run_repl() {
    use std::io::{self, Write};
    eprintln!("XIOM REPL v{} -- type :help for commands, :quit to exit", env!("CARGO_PKG_VERSION"));
    let mut line_num = 0u64;
    let mut state: Vec<String> = Vec::new(); // accumulated let/var declarations
    // Unique per REPL session: two REPLs must not collide on `_repl_1.xi`.
    let repl_suffix = xiom::jit::rand_suffix();

    loop {
        line_num += 1;
        print!("xiom> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // Special commands
        if trimmed.starts_with(':') {
            match trimmed {
                ":quit" | ":q" => break,
                ":help" | ":h" => {
                    eprintln!("  :quit, :q    Exit the REPL");
                    eprintln!("  :help, :h    Show this help");
                    eprintln!("  :vars        Show accumulated variables");
                    eprintln!("  :reset       Clear accumulated state");
                    eprintln!("  :type <e>    Show the type of an expression (future)");
                    eprintln!("  Any other input is compiled as a script and executed.");
                }
                ":vars" => {
                    if state.is_empty() { eprintln!("  (no variables)"); }
                    else { for v in &state { eprintln!("  {v}"); } }
                }
                ":reset" => { state.clear(); eprintln!("  State cleared."); }
                _ if trimmed.starts_with(":type") => {
                    eprintln!("  (type inspection coming in REPL v2)");
                }
                _ => eprintln!("  Unknown command: {trimmed}. Type :help for commands."),
            }
            continue;
        }

        // Track let/var declarations for state persistence
        let is_binding = trimmed.starts_with("let ") || trimmed.starts_with("var ");
        if is_binding {
            state.push(trimmed.to_string());
        }

        // Build source: accumulated state + current input
        let mut source = String::new();
        for stmt in &state {
            source.push_str(stmt);
            source.push('\n');
        }
        source.push_str(trimmed);
        source.push('\n');

        let source = xiom::implicit_main::wrap_implicit_main(&source);
        let tmp_dir = std::env::temp_dir().join("xiom_repl");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let tmp_src = tmp_dir.join(format!("_repl_{repl_suffix:x}_{line_num}.xi"));
        let tmp_out = tmp_dir.join(format!("_repl_{repl_suffix:x}_{line_num}.exe"));
        std::fs::write(&tmp_src, &source).ok();

        let config = CompileConfig {
            output_file: Some(tmp_out.to_string_lossy().to_string()),
            do_run: true,
            ..CompileConfig::default()
        };
        let sources = vec![tmp_src.to_string_lossy().to_string()];
        if let Err(errors) = compile(&config, &sources) {
            for e in &errors { eprintln!("error: {e}"); }
        }
    }
    eprintln!("Goodbye.");
}

fn main() {
    // Deep recursion (parser depth cap 128, codegen visitors) needs far more
    // than the default 1MB Windows main-thread stack. rustc runs its compiler
    // on a big-stack thread for the same reason. All work happens here;
    // process::exit calls inside real_main still terminate immediately.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(real_main)
        .expect("failed to spawn compiler worker thread");
    if child.join().is_err() {
        // A panic escaped real_main; mirror the standard panic exit code.
        std::process::exit(101);
    }
}

fn real_main() {
    // Audit #12: the timeout / memory watchdogs set this cooperative token
    // instead of calling process::exit from a worker thread.
    xiom_codegen::cancel::reset();
    // Stage 5: clap owns the flag surface (src/cli.rs). It parses for
    // validation and returns argv unchanged, so every legacy scan below is
    // byte-compatible; the reads migrate to the clap matches in follow-up
    // work without touching the surface definition.
    let args = cli::parse(env::args().collect());
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(if args.iter().any(|a| a == "--help") { 0 } else { 1 });
    }

    // Show AI help
    if args.iter().any(|a| a == "--help-ai") {
        eprintln!("{}", xiom::ai::ai_help_text());
        process::exit(0);
    }

    if args.iter().any(|a| a == "--version") {
        let tag = option_env!("XIOM_RELEASE_TAG").unwrap_or("Production Polish");
        let stats = option_env!("XIOM_RELEASE_STATS").unwrap_or("27/27 E2E core, 516+ unit tests, 19/19 gates");
        println!("XIOM Compiler v{} \"{tag}\" - {stats}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // 5c-R: --explain EXXXX opens the error code reference
    if let Some(pos) = args.iter().position(|a| a == "--explain") {
        if let Some(code) = args.get(pos + 1) {
            if !xiom::explain_error(code) {
                process::exit(1);
            }
            return;
        }
        eprintln!("usage: xiom --explain <code>  (e.g., xiom --explain X0010)");
        process::exit(1);
    }

    // v0.55: xiom build-runtime -- pre-compile C runtime shared library for OrcJIT
    if args.get(1).map_or(false, |a| a == "build-runtime") {
        let output_dir = xiom::jit::jit_cache_dir();
        match xiom_jit::build_runtime_library(&output_dir) {
            Ok(path) => {
                eprintln!("  Runtime library built: {}", path.display());
                eprintln!("  OrcJIT ready: use --jit flag for in-process compilation.");
            }
            Err(e) => {
                eprintln!("error: runtime build failed: {e}");
                process::exit(1);
            }
        }
        return;
    }

    // -- M10.4: xiom repl -- interactive scripting shell --------------
    if args.get(1).map_or(false, |a| a == "repl") {
        run_repl();
        return;
    }

    // -- M10: xiom run -- JIT/scripting execution ---------------------
    if args.get(1).map_or(false, |a| a == "run") {
        let remaining: Vec<&str> = args.iter().skip(2).map(|s| s.as_str()).collect();
        if remaining.is_empty() {
            eprintln!("usage: xiom run <file.xi>     execute a script");
            eprintln!("       xiom run -e \"<code>\"   execute inline code");
            eprintln!("       xiom run -               read script from stdin");
            eprintln!("       xiom run --watch <file>  watch and re-run on changes");
            process::exit(1);
        }

        let watch_mode = remaining.contains(&"--watch");
        let effective: Vec<&str> = remaining.iter()
            .filter(|&&a| a != "--watch" && a != "--cache" && a != "--no-cache" && a != "--jit")
            .copied().collect();
        if effective.is_empty() { process::exit(1); }

        let source = if effective[0] == "-e" {
            // xiom run -e "expr"
            if effective.len() < 2 {
                eprintln!("error: -e requires an expression");
                process::exit(1);
            }
            effective[1..].join(" ")
        } else if effective[0] == "-" {
            // xiom run -  (read from stdin)
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error reading stdin: {e}"); process::exit(1);
            });
            buf
        } else {
            // xiom run <file.xi>  (possibly with --watch)
            let path = effective[0];
            if watch_mode {
                // Watch mode: compile once, then poll for changes
                run_script_watch(path);
                return;
            }
            match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: cannot read '{path}': {e}"); process::exit(1); }
            }
        };

        // Apply implicit main wrapping for scripting convenience
        let source = xiom::implicit_main::wrap_implicit_main(&source);

        // M10: Check script cache for instant re-run
        let use_jit = effective.contains(&"--jit");
        let no_cache = effective.contains(&"--no-cache");
        if !no_cache {
            if let Some(cached) = xiom::jit::script_cache_get(&source) {
                let output = std::process::Command::new(&cached).output();
                if let Ok(out) = output {
                    if out.status.success() && !use_jit {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if !stdout.is_empty() { print!("{stdout}"); }
                        return;
                    }
                }
            }
        }

        // M10.1d: True JIT execution if --jit flag is set
        if use_jit {
            match xiom::jit::jit_execute(&source) {
                Ok(code) => { eprintln!("  JIT exit code: {code}"); return; }
                Err(e) => { eprintln!("  JIT error: {e}"); process::exit(1); }
            }
        }

        // Write to temp file, compile, and run
        let tmp_dir = std::env::temp_dir().join("xiom_run");
        let _ = std::fs::create_dir_all(&tmp_dir);
        // Unique per invocation: concurrent `xiom run` processes must not
        // race on a shared `_script.xi` / `_script.exe`.
        let run_suffix = xiom::jit::rand_suffix();
        let tmp_src = tmp_dir.join(format!("_script_{run_suffix:x}.xi"));
        let tmp_out = tmp_dir.join(format!("_script_{run_suffix:x}.exe"));
        std::fs::write(&tmp_src, &source).unwrap_or_else(|e| {
            eprintln!("error: cannot write temp file: {e}"); process::exit(1);
        });

        // Resolve runtime libraries needed for linking using project discovery
        let (_, graph_dirs) = xiom::expand_sources_with_graph(&[tmp_src.to_string_lossy().to_string()]);
        let mut link_paths: Vec<String> = graph_dirs.into_iter().collect();
        // Add CARGO_MANIFEST_DIR-based runtime path as fallback
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let runtime = std::path::Path::new(&manifest).join("..").join("..").join("runtime");
            if runtime.is_dir() {
                link_paths.push(runtime.to_string_lossy().to_string());
            }
        }
        let link_libs: Vec<String> = Vec::new();
        let c_sources: Vec<String> = Vec::new(); // Auto-discovered by compile()

        // M12: Ensure XIOM_STDLIB is set for scripting mode so stdlib types resolve
        if std::env::var("XIOM_STDLIB").is_err() {
            // Try to find stdlib relative to the xiom binary
            let exe = std::env::current_exe().unwrap_or_default();
            let mut search = exe.parent();
            for _ in 0..8 {
                if let Some(dir) = search {
                    let candidate = dir.join("stdlib");
                    if candidate.is_dir() {
                        // SAFETY: set_var is called before any threads are spawned
                        unsafe { std::env::set_var("XIOM_STDLIB", candidate.to_string_lossy().to_string()); }
                        break;
                    }
                    search = dir.parent();
                } else { break; }
            }
        }

        let config = CompileConfig {
            output_file: Some(tmp_out.to_string_lossy().to_string()),
            do_run: true,
            script_mode: true,
            link_paths,
            link_libs,
            c_sources,
            ..CompileConfig::default()
        };

        let sources = vec![tmp_src.to_string_lossy().to_string()];
        compile_or_exit(&config, &sources);

        // M10: Cache the compiled script for instant re-run
        if !no_cache {
            xiom::jit::script_cache_put(&source, &tmp_out);
        }
        return;
    }

    // Stage 5 (clap step 2): flag reads below come from the clap matches;
    // exact-form-sensitive checks and command words use the raw view.
    let emit_ir = args.flag("emit-ir");
    let emit_tokens = args.flag("emit-tokens");
    let do_run = args.flag("run");
    let check_only = args.flag("check");
    let release = args.flag("release");
    // 2026-09-10: explicit optimization level (--opt-level 0..=3). The old
    // -O2 floor existed because pre-CRT-layout IR miscompiled at -O0/-O1
    // (i128 loops + inlined Vec ops); the floor is now overridable.
    let opt_level: Option<u8> = args.value("opt-level")
        .and_then(|v| v.parse::<u8>().ok());
    let target = parse_target(args.value("target").as_deref());
    let check_contracts = !args.flag("no-contracts") && !release;
    let runtime_contracts = args.flag("runtime-contracts");
    // Security review (2026-08-13): release builds strip assert/dbg!/debugger;
    // --keep-debug-checks retains them in release binaries.
    let keep_debug_checks = args.flag("keep-debug-checks");
    // Exact `--diagnostics=json` form only (the space form was never read).
    let diagnostics_json = args.raw_has("--diagnostics=json");
    let strict_mode = args.flag("strict");
    let debug_symbols = args.flag("debug") || args.flag("debug-symbols");
    let shared_lib = args.flag("shared");
    let static_lib = args.flag("static");
    let watch_mode = args.flag("watch");
    let hot_reload = args.flag("hot-reload");
    let hot_reload_contracts = args.flag("hot-reload-contracts");
    // v0.55: OrcJIT -- in-process JIT compilation via clang DLL loading
    let _use_jit = args.flag("jit");
    // v0.56: Lazy JIT -- incremental recompilation (only with --jit)
    let _lazy_jit = args.flag("lazy");
    // v0.56: LTO -- ThinLTO link-time optimization
    let use_lto = args.flag("lto");
    // 7E.1: Sanitizer flags
    let sanitize: Option<String> = args.value("sanitize");
    // 7E.2: Stack protector
    let stack_protector = args.flag("stack-protector");
    let overflow_checks = args.flag("overflow-checks");
    let strict_exhaustive = args.flag("strict-exhaustive");
    // D2.1 (Phase 7): allow `#[unsafe_direct]` (trusted escape hatch) in user code
    let enable_unsafe_direct = args.flag("enable-unsafe-direct");
    if enable_unsafe_direct {
        // Security review (2026-08-13): the escape hatch disables the
        // unsafe-confinement guard -- surface it loudly on every invocation so
        // a release build log cannot silently contain unguarded code.
        eprintln!(
            "warning: --enable-unsafe-direct is set -- #[unsafe_direct] code bypasses \
             the unsafe-confinement fault guard; audit any binary built with this flag"
        );
    }
    // v0.54: Binary cache -- cache compiled binary by SHA-256 source hash
    let use_cache = args.flag("cache");
    // 5e.5f: Incremental compilation flags
    let incremental = args.flag("incremental");
    let force_recompile = args.flag("force");
    // 7C: Parallel compilation flags
    let parallel = args.flag("parallel") && !args.flag("sequential");
    // I2: Parallel codegen -- rayon-based per-function IR emission
    let parallel_codegen = args.flag("parallel-codegen");
    let jobs: usize = args.value("jobs")
        .and_then(|v| v.parse().ok()).unwrap_or(0);
    // 5g AI Pipeline flags
    let ai_mode = args.flag("ai");
    let ai_local = args.flag("ai-local");
    let ai_dry_run = args.flag("ai-dry-run");
    let ai_silent = args.flag("ai-silent");
    let ai_strict = args.flag("ai-strict");
    let _ai_batch = args.flag("ai-batch") || args.flag("batch");
    let ai_model: Option<String> = args.value("ai-model");
    let ai_timeout: Option<u32> = args.value("ai-timeout")
        .and_then(|v| v.parse().ok());
    let test_mode = args.flag("test");
    let clean_mode = args.flag("clean");
    let install_mode = args.raw_has("install");
    let install_pkg = args.iter().position(|a| a == "install")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|p| !p.starts_with('-'));
    let publish_mode = args.raw_has("publish");
    let update_mode = args.raw_has("update");
    let bench_mode = args.raw_has("bench");
    let bench_count: u32 = args.value("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let init_mode = args.raw_has("init");
    let new_mode = args.raw_has("new");
    let build_mode = args.raw_has("build");
    let doctor_mode = args.raw_has("doctor") || args.flag("doctor");
    let doc_mode = args.raw_has("doc") || args.flag("doc");
    let graph_mode = args.present("graph");
    // Exact-form checks: only `--graph=mermaid`/`--graph-format=mermaid`
    // select a format; bare `--graph` defaults to dot (legacy contract).
    let graph_format = if args.raw_has("--graph=mermaid") || args.raw_has("--graph-format=mermaid") {
        Some("mermaid")
    } else if args.raw_has("--graph=dot") || args.raw_has("--graph-format=dot") {
        Some("dot")
    } else if graph_mode {
        Some("dot") // default
    } else {
        None
    };
    let new_name: Option<String> = args.iter().position(|a| a == "new")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|n| !n.starts_with('-'));
    let registry_cmd = args.raw_has("registry");

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
        let clean_cache = args.flag("cache");
        if clean_cache {
            let _ = xiom::jit::cache_clean();
            return;
        }
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
        let registry_url = args.value("registry")
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

    if doctor_mode {
        run_doctor();
        return;
    }

    if doc_mode {
        run_doc(&args);
        return;
    }

    let verify = args.flag("verify") || args.raw_has("--verify-output");
    let verify_output = args.value("verify-output");

    let output_file = args.value("output");

    let link_libs = args.values("link");
    let link_paths = args.values("link-path");
    let c_sources = args.values("c-source");

    // Resolved early so the timeout watchdog can consult the project
    // manifest ([compiler] timeout-secs) before spawning.
    let source_paths = resolve_source_files(&args);

    let timeout_secs: u64 = args.value("timeout")
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            // AUDIT #18 (completion): xiom.toml [compiler] timeout-secs is a
            // project default (explicit --timeout wins). Honored now that the
            // watchdog uses cooperative cancellation instead of a worker
            // thread process::exit (audit #12).
            source_paths.first().and_then(|p| {
                xiom_graph::manifest::find_and_parse_manifest(std::path::Path::new(p))
                    .ok()
                    .and_then(|m| m.compiler.timeout_secs)
                    .map(|t| t as u64)
            })
        })
        .unwrap_or(300);

    let max_recursion_depth: u32 = args.value("max-depth")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let max_recursion_depth = std::cmp::min(max_recursion_depth, 10000u32);

    if timeout_secs > 0 {
        let duration = Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            eprintln!("error: compilation timed out after {} seconds", timeout_secs);
            // Audit #12: cooperative cancellation -- codegen observes this
            // between functions and the clang child is killed; real_main
            // reports the failure and exits from the MAIN thread.
            xiom_codegen::cancel::request_cancel();
        });
    }

    let max_memory_mb: u64 = args.value("max-memory-mb")
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
                        // Audit #12: cooperative cancellation (see timeout).
                        xiom_codegen::cancel::request_cancel();
                        return;
                    }
                }
            }
        });
    }

    let source_paths = resolve_source_files(&args);

    // -- M10.2: xiom build --standalone -- script-to-binary ----------
    let standalone_mode = args.flag("standalone");
    let scaffold_mode = args.flag("scaffold");

    if standalone_mode && !source_paths.is_empty() {
        let script_path = &source_paths[0];
        let source = match std::fs::read_to_string(script_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error: cannot read '{script_path}': {e}"); process::exit(1); }
        };
        let wrapped = xiom::implicit_main::wrap_implicit_main(&source);
        let out_name = output_file.clone().unwrap_or_else(|| {
            let stem = std::path::Path::new(script_path)
                .file_stem().and_then(|s| s.to_str()).unwrap_or("script");
            if cfg!(windows) { format!("{stem}.exe") } else { stem.to_string() }
        });

        if scaffold_mode {
            let proj_name = std::path::Path::new(script_path)
                .file_stem().and_then(|s| s.to_str()).unwrap_or("script");
            let proj_dir = std::path::Path::new(&proj_name);
            let src_dir = proj_dir.join("src");
            std::fs::create_dir_all(&src_dir).unwrap_or_else(|e| {
                eprintln!("error: cannot create project dir: {e}"); process::exit(1);
            });
            std::fs::write(src_dir.join("main.xi"), &wrapped).unwrap_or_else(|e| {
                eprintln!("error: cannot write main.xi: {e}"); process::exit(1);
            });
            std::fs::write(proj_dir.join("package.xi"), format!(
                "[package]\nname = \"{proj_name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n"
            )).unwrap_or_else(|e| {
                eprintln!("error: cannot write package.xi: {e}"); process::exit(1);
            });
            eprintln!("  Scaffolded project: {proj_name}/");
        }

        let config = CompileConfig {
            output_file: Some(out_name.clone()),
            release: true,
            ..CompileConfig::default()
        };
        let tmp_src = std::env::temp_dir()
            .join("xiom_standalone")
            .join(format!("_script_{:x}.xi", xiom::jit::rand_suffix()));
        let _ = std::fs::create_dir_all(tmp_src.parent().unwrap());
        std::fs::write(&tmp_src, &wrapped).unwrap_or_else(|e| {
            eprintln!("error: cannot write temp file: {e}"); process::exit(1);
        });
        compile_or_exit(&config, &[tmp_src.to_string_lossy().to_string()]);
        eprintln!("  Standalone binary: {out_name}");
        return;
    }

    if standalone_mode && source_paths.is_empty() {
        eprintln!("usage: xiom build --standalone [--scaffold] <script.xi> [-o output]");
        eprintln!("  Converts a script into a standalone production binary.");
        process::exit(1);
    }

    if source_paths.is_empty() && !test_mode && !build_mode && !graph_mode {
        eprintln!("error: no source file(s) provided");
        process::exit(1);
    }

    if test_mode {
        run_xiom_tests(&args);
        return;
    }

    let config = {
        let mut config = CompileConfig {
            target,
            emit_ir,
            do_run,
            check_only,
            release,
            opt_level,
            check_contracts,
            diagnostics_json,
            strict_mode,
            debug_symbols,
            shared_lib,
            static_lib,
            max_recursion_depth,
            dump_contracts: args.flag("dump-contracts"),
            verify,
            verify_output,
            output_file,
            link_libs,
            link_paths,
            c_sources,
        hot_reload: false,  // set to true by hot reload loop below
        hot_reload_contracts,
        sanitize,
        stack_protector,
        runtime_contracts,
        keep_debug_checks,
        overflow_checks,
        strict_exhaustive,
        incremental,
        force: force_recompile,
        parallel,
        jobs,
        script_mode: false,
        cache: use_cache,
        lto: use_lto,
        parallel_codegen,
        enable_unsafe_direct,
    };

    // AUDIT #18 FIX (second half): the xiom.toml `[compiler]` table was
    // PARSED but IGNORED -- project settings never reached compilation.
    // Manifest values now act as PROJECT DEFAULTS; explicit CLI flags keep
    // precedence (a flag absent = field not chosen by the user).
    if let Some(first_src) = source_paths.first() {
        if let Ok(manifest) = xiom_graph::manifest::find_and_parse_manifest(
            std::path::Path::new(first_src))
        {
            let cc = &manifest.compiler;
            if cc.release && !release {
                config.release = true;
            }
            if cc.incremental && !incremental {
                config.incremental = true;
            }
            if let Some(d) = cc.max_depth {
                if args.value("max-depth").is_none() {
                    config.max_recursion_depth = d;
                }
            }
            if let Some(t) = &cc.target {
                if args.value("target").is_none() {
                    match t.as_str() {
                        "wasm" => config.target = Target::Wasm,
                        "wasi" => config.target = Target::Wasi,
                        "arm" => config.target = Target::Arm,
                        "riscv" => config.target = Target::RisCv,
                        "native" => config.target = Target::Native,
                        other => eprintln!("warning: ignoring unknown [compiler] target '{other}'"),
                    }
                }
            }
            if std::env::var("XIOM_VERBOSE_CONFIG").as_deref() == Ok("1") {
                eprintln!("[config] manifest applied: release={} incremental={} max_depth={:?}",
                    config.release, config.incremental, cc.max_depth);
            }
        }
    }
    config
    };

    // 7F.2: Build graph visualization
    if let Some(fmt) = graph_format {
        let first = if !source_paths.is_empty() {
            std::path::Path::new(&source_paths[0]).to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        };
        match xiom_graph::build_project_graph(&first) {
            Ok(graph) => {
                let format = if fmt == "mermaid" {
                    xiom::graph_viz::GraphFormat::Mermaid
                } else {
                    xiom::graph_viz::GraphFormat::Dot
                };
                let output = xiom::graph_viz::generate_dot_graph(&graph, format);
                println!("{output}");
            }
            Err(e) => {
                eprintln!("error: cannot build dependency graph: {e}");
                process::exit(1);
            }
        }
        return;
    }

    // --emit-tokens: output token stream and exit
    if emit_tokens && !source_paths.is_empty() {
        for path_str in &source_paths {
            let source = match std::fs::read_to_string(path_str) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: {}: {}", path_str, e); continue; }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            for tok in &tokens {
                println!("{}:{}: {:?}", tok.span.line, tok.span.col, tok.kind);
            }
        }
        return;
    }

    // 7F.1: Build daemon mode
    if build_mode && !watch_mode {
        if !source_paths.is_empty() {
            let (resolved, _) = xiom::expand_sources_with_graph(&source_paths);
            compile_or_exit(&config, &resolved);
        } else {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            if let Ok(graph) = xiom_graph::build_project_graph(&cwd) {
                eprintln!("Build: {} ({} modules)", graph.project_name, graph.len());
                match graph.compilation_order() {
                    Ok(order) => {
                        let files: Vec<String> = order.iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        compile_or_exit(&config, &files);
                    }
                    Err(e) => { eprintln!("error: {e}"); process::exit(1); }
                }
            } else {
                eprintln!("error: no xiom.toml or package.xi found. Run 'xiom init' first.");
                process::exit(1);
            }
        }
        return;
    }

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
            // AUDIT #11 FIX: unreadable input used to flow through
            // unwrap_or_default() as an EMPTY program -- the audit then
            // scored nothing and the process exited 0 (false-green CI).
            let source = match std::fs::read_to_string(source_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot read '{source_path}': {e}");
                    overall_exit = std::cmp::max(overall_exit, 3);
                    continue;
                }
            };
            let mut parser = Parser::new(Lexer::new(&source).tokenize());
            // AUDIT #11 FIX (part 2): parse failures were silently skipped
            // via `if let Ok`, and partial-AST errors were never taken.
            let program = match parser.parse_program() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {e}");
                    overall_exit = std::cmp::max(overall_exit, 3);
                    continue;
                }
            };
            let parse_errors = parser.take_errors();
            if !parse_errors.is_empty() {
                for e in &parse_errors {
                    eprintln!("parse error: {e:?}");
                }
                overall_exit = std::cmp::max(overall_exit, 3);
                continue;
            }
            {
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
            hot_reload,
            incremental,
            force: force_recompile,
            ..config
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

        // Initial compile -- exit on failure for hot reload
        if let Err(errors) = compile(&hot_config, &source_paths) {
            for e in &errors { eprintln!("error: {e}"); }
            process::exit(1);
        }

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
            if changed { if let Err(errors) = compile(&hot_config, &source_paths) { for e in &errors { eprintln!("error: {e}"); } } }
        }
    }

    // 5g AI Pipeline: run check-only compile first to get diagnostics, then call LLM
    if ai_mode || ai_local || ai_dry_run {
        let ai_config = xiom::ai::load_ai_config(ai_model.clone());
        let mut ai_config = xiom::ai::AiConfig {
            enabled: true, local_only: ai_local, dry_run: ai_dry_run,
            silent: ai_silent, strict: ai_strict,
            timeout_secs: ai_timeout.unwrap_or(ai_config.timeout_secs),
            model: ai_model.unwrap_or(ai_config.model),
            ..ai_config
        };
        // AI-05: enforce the --ai-local privacy contract and refuse plaintext
        // key transmission before any request.
        if let Err(e) = xiom::ai::finalize_config(&mut ai_config) {
            eprintln!("[AI] {e}");
            process::exit(1);
        }
        eprintln!("[AI] Provider: {}, Model: {}",
            ai_config.provider, ai_config.model);

        // 5f.3e: Batch mode -- collect diagnostics from all files into single output
        if _ai_batch {
            let mut all_diagnostics: Vec<xiom::Diagnostic> = Vec::new();
            let mut all_sources: Vec<(String, String)> = Vec::new(); // (path, source)

            for path in &source_paths {
                let check_config = CompileConfig {
                    check_only: true, emit_ir: true, diagnostics_json: true,
                    target: config.target, release: config.release,
                    do_run: false, check_contracts: config.check_contracts,
                    strict_mode: config.strict_mode, debug_symbols: config.debug_symbols,
                    shared_lib: false, static_lib: false,
                    max_recursion_depth: config.max_recursion_depth,
                    dump_contracts: config.dump_contracts,
                    verify: config.verify,
                    verify_output: config.verify_output.clone(),
                    output_file: None,
                    incremental: false, force: false,
                    parallel: false, jobs: 0,
                    link_libs: vec![],
                    link_paths: vec![],
                    c_sources: vec![],
                    hot_reload: false,
                    hot_reload_contracts: false,
                    sanitize: None,
                    stack_protector: false,
                    runtime_contracts: false,
                    keep_debug_checks: false,
                    overflow_checks: config.overflow_checks,
                    strict_exhaustive: config.strict_exhaustive,
                    script_mode: false,
                    cache: false,
                    lto: false,
                    parallel_codegen: false,
                    enable_unsafe_direct: enable_unsafe_direct,
                    opt_level: config.opt_level,
                };
                let result = xiom::compile_with_diagnostics(&check_config, &[path.clone()]);
                let source = std::fs::read_to_string(path).unwrap_or_default();
                all_diagnostics.extend(result.diagnostics);
                all_sources.push((path.clone(), source));
            }

            if !all_diagnostics.is_empty() {
                // 5f.3f: Run Z3 verification for contract violations to get counterexamples
                let z3_models = xiom::ai::run_z3_for_contract_errors(&all_diagnostics, &all_sources);
                match xiom::ai::run_ai_pipeline_batch(&ai_config, &all_sources, &all_diagnostics, &z3_models) {
                    Ok(output) if !ai_silent => {
                        eprintln!("xiom --ai --batch: {} hints -> .xiom_ai.json ({} API, {} cached, {} Z3 models)",
                            output.total_hints, output.api_calls, output.cached_hints, z3_models.len());
                    }
                    Err(e) => eprintln!("[AI] {e}"),
                    _ => {}
                }
            } else {
                eprintln!("[AI] All sources compile cleanly -- no diagnostics.");
            }
        } else {
            // Single-file mode (existing behavior)
            for path in &source_paths {
                let check_config = CompileConfig {
                    check_only: true, emit_ir: true, diagnostics_json: true,
                    target: config.target, release: config.release,
                    do_run: false, check_contracts: config.check_contracts,
                    strict_mode: config.strict_mode, debug_symbols: config.debug_symbols,
                    shared_lib: false, static_lib: false,
                    max_recursion_depth: config.max_recursion_depth,
                    dump_contracts: config.dump_contracts,
                    verify: config.verify,
                    verify_output: config.verify_output.clone(),
                    output_file: None,
                    incremental: false, force: false,
                    parallel: false, jobs: 0,
                    link_libs: vec![],
                    link_paths: vec![],
                    c_sources: vec![],
                    hot_reload: false,
                    hot_reload_contracts: false,
                    sanitize: None,
                    stack_protector: false,
                    runtime_contracts: false,
                    keep_debug_checks: false,
                    overflow_checks: config.overflow_checks,
                    strict_exhaustive: config.strict_exhaustive,
                    script_mode: false,
                    cache: false,
                    lto: false,
                    parallel_codegen: false,
                    enable_unsafe_direct: enable_unsafe_direct,
                    opt_level: config.opt_level,
                };
                let result = xiom::compile_with_diagnostics(&check_config, &[path.clone()]);
                let source = std::fs::read_to_string(path).unwrap_or_default();

                // 5f.3f: Run Z3 for contract errors in single-file mode too
                let diags = result.diagnostics.clone();
                let sources = vec![(path.clone(), source.clone())];
                let z3_models = xiom::ai::run_z3_for_contract_errors(&diags, &sources);

                if !result.diagnostics.is_empty() {
                    match xiom::ai::run_ai_pipeline(&ai_config, &source, path, &result.diagnostics, &z3_models) {
                        Ok(output) if !ai_silent => {
                            eprintln!("xiom --ai: {} hints -> .xiom_ai.json ({} API, {} cached, {} Z3 models)",
                                output.total_hints, output.api_calls, output.cached_hints, z3_models.len());
                        }
                        Err(e) => eprintln!("[AI] {e}"),
                        _ => {}
                    }
                } else {
                    eprintln!("[AI] No diagnostics -- source compiles cleanly.");
                }
            }
        }
    }

    compile_or_exit(&config, &source_paths);
}

fn print_usage() {
        let tag = option_env!("XIOM_RELEASE_TAG").unwrap_or("Production Polish");
        let stats = option_env!("XIOM_RELEASE_STATS").unwrap_or("27/27 E2E core, 516+ unit tests, 19/19 gates");
        eprintln!("XIOM Compiler v{} \"{tag}\" -- {stats}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom [OPTIONS] <source.xi>");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help              Show this help message");
    eprintln!("  --version           Print version");
    eprintln!("  -o <output>         Output binary path (default: a.exe on Windows, a.out on Linux/macOS)");
    eprintln!("  --run               Compile and run (requires fn main())");
    eprintln!("  run <file.xi>       Execute as script (auto-wraps in fn main())");
    eprintln!("  run -               Read script from stdin");
    eprintln!("  run -e \"<code>\"     Execute inline code");
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
    eprintln!("  --hot-reload-contracts  7D: Verify contracts before hot-swapping function pointers");
    eprintln!("  --sanitize=<type>    7E.1: Enable sanitizer (address, undefined, leak, thread)");
    eprintln!("  --stack-protector    7E.2: Enable stack canaries (-fstack-protector)");
    eprintln!("  --overflow-checks    Enable integer overflow runtime checks");
    eprintln!("  --keep-debug-checks  Keep debug intrinsics (assert/dbg!/todo!/debugger) in release builds");
    eprintln!("  --enable-unsafe-direct  D2.1: allow #[unsafe_direct] (trusted escape hatch) in user code");
    eprintln!("  --graph             7F.2: Output dependency graph (DOT format)");
    eprintln!("  --graph=mermaid     7F.2: Output dependency graph (Mermaid format)");
    eprintln!("  build               7F.1: Build entire project (from xiom.toml)");
    eprintln!("  build --watch       7F.1: Build daemon -- watch and rebuild on changes");
    eprintln!("  --runtime-contracts  7E.4: Force runtime contract checks (even in release mode)");
    eprintln!("  --no-contracts       Disable all contract checks (faster, less safe)");
    eprintln!("  --incremental       5e.5f: Cache compiled IR, skip unchanged sources");
    eprintln!("  --force             5e.5f: Force recompile -- ignore all caches");
    eprintln!("  --parallel          7C: Enable parallel lex+parse (rayon thread pool)");
    eprintln!("  --sequential        7C: Force sequential compilation (disable parallel)");
    eprintln!("  --jobs <N>          7C: Number of parallel compile jobs (default: num CPUs)");
    eprintln!("  --ai                AI-assisted diagnostics (requires Ollama or API key)");
    eprintln!("  --ai-local          AI mode: local LLM only, never sends code off-machine");
    eprintln!("  --ai-dry-run        AI mode: print prompt, don't call LLM");
    eprintln!("  --ai-strict         Refuse binary output on any contract violation");
    eprintln!("  --ai-batch          Batch mode: analyze all source files, single .xiom_ai.json");
    eprintln!("  --ai-model=<name>   Override AI model (default: codellama)");
    eprintln!("  --ai-timeout=<sec>  AI LLM call timeout (default: 10s)");
    eprintln!("  --help-ai           Show AI mode setup and configuration guide");
    eprintln!("  --timeout <seconds>  Set compilation timeout (default: 300; 0 disables)");
    eprintln!("  --max-memory-mb <N>       Set max memory budget in MB (0 = disabled)");
    eprintln!("  --link <name>             Link a native library (repeatable, e.g. vulkan-1)");
    eprintln!("  --link-path <dir>         Add a library search path (repeatable, -L<dir>)");
    eprintln!("  --c-source <file>         Link an extra C/object file (repeatable)");
    eprintln!();
    eprintln!("MORE OPTIONS (advanced):");
    eprintln!("  --check             Type-check only (no code generation)");
    eprintln!("  --emit-tokens       Print the token stream and exit");
    eprintln!("  --debug, -g         Emit debug info");
    eprintln!("  --release           Release build (strips contracts and debug intrinsics)");
    eprintln!("  --opt-level <0..3>  Explicit optimization level");
    eprintln!("  --lto               ThinLTO link-time optimization");
    eprintln!("  --cache / --no-cache  Binary cache by source hash (default: on)");
    eprintln!("  --jit               In-process JIT execution (clang DLL backend)");
    eprintln!("  --lazy              Incremental recompilation (with --jit)");
    eprintln!("  --parallel-codegen  Rayon-based per-function IR emission");
    eprintln!("  --strict            Strict mode (alias: --strict-mode)");
    eprintln!("  --strict-exhaustive Strict exhaustiveness checks");
    eprintln!("  --static            Compile as a static library");
    eprintln!("  --standalone        Standalone build (no stdlib runtime dependency)");
    eprintln!("  --max-depth <N>     Max contract recursion depth (default: 500)");
    eprintln!("  --count <N>         Iterations for `xiom bench` (default: 10)");
    eprintln!("  --bench-file <f>    Benchmark a single file");
    eprintln!("  --test-dir <dir>    Test directory for `xiom --test` (default: examples)");
    eprintln!("  --test              Run the example test suite");
    eprintln!("  --scaffold          With `build --standalone`: scaffold a project from a script");
    eprintln!("  --clean             Clean build artifacts and caches");
    eprintln!("  --explain <CODE>    Explain a diagnostic code (e.g. --explain T001)");
    eprintln!("  --registry <url>    Package registry URL (pkg)");
    eprintln!("  --locked            Use the lockfile exactly (pkg)");
    eprintln!("  --frozen            Offline lockfile-only mode (pkg)");
    eprintln!("  --ai-silent         With --ai: suppress stdout, write only .xiom_ai.json");
    eprintln!("  --ai-batch, --batch With --ai: analyze all sources, single .xiom_ai.json");
    eprintln!();
    eprintln!("SUBCOMMANDS:");
    eprintln!("  repl                Interactive REPL");
    eprintln!("  build-runtime       Build the C runtime library");
    eprintln!("  doc <file.xi>       Generate documentation (Markdown/HTML)");
    eprintln!("  doctor              Check toolchain dependencies and report status");
    eprintln!();
    eprintln!("TOOL DISPATCHERS (launcher script: xiom.bat / xiom wrapper):");
    eprintln!("  fmt | lsp | mcp | pkg | dbg | verify | ffigen | ai | graph | test");
    eprintln!("  Each runs the matching xiom-* binary from the same bin/ directory.");
    eprintln!();
    eprintln!("DEPENDENCIES:");
    eprintln!("  Required: clang (LLVM) -- to compile IR to native binary");
    eprintln!("  Optional: opt (LLVM) -- IR optimization pass (-O1)");
    eprintln!("  Optional: nasm -- hardware-accelerated crypto/memcpy (stdlib)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom --run examples/demo_float.xi");
    eprintln!("  xiom -o prog.exe source.xi");
    eprintln!("  xiom --emit-ir examples/demo_float.xi");
    eprintln!("  xiom --target wasm -o prog.wasm source.xi");
    eprintln!("  xiom --verify examples/phase1_contracts.xi");
}

fn parse_target(target: Option<&str>) -> Target {
    match target {
        Some("wasm") => Target::Wasm,
        Some("wasi") => Target::Wasi,
        Some("arm") => Target::Arm,
        Some("riscv") => Target::RisCv,
        _ => Target::Native,
    }
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

fn run_xiom_tests(args: &cli::Cli) {
    use std::process::Command as Cmd;

    // Legacy contract: `--test <dir>` used the token after `--test` as the
    // directory (even though `--test` is also a bool flag); `--test-dir` is
    // the explicit spelling. Kept on the raw view to preserve that quirk.
    let test_dir = args.iter().position(|a| a == "--test")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| args.value("test-dir"))
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
        let xiompath = std::env::current_exe().unwrap_or_else(|_| "xiom".into());
        let compile = Cmd::new(&xiompath)
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

// -- Phase 5d: Package Manager ------------------------------------------

fn handle_install(_args: &[String], pkg_name: Option<&str>, registry_url: &str, _update: bool) {
    // CRB-3c wrinkle: install into the SAME tree as `xiom pkg`
    // (`<XIOM_HOME>/packages`), not the old `~/.xiom/packages`, so both
    // spellings see one another. `xiom pkg` remains the supported path.
    let pkgs_dir = xiom_graph::paths::xiom_home().join("packages");
    let pkgs_dir = pkgs_dir.to_string_lossy().to_string();
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
    // CRB-3c wrinkle: the legacy registry list now lives under XIOM_HOME
    // (`<home>/registry.json`) so `xiom install/registry` and `xiom pkg`
    // share one tree; an existing `~/.xiom/registry.json` is still honored
    // so old setups keep working.
    let home = xiom_graph::paths::xiom_home();
    let new_path = home.join("registry.json");
    let legacy_path = dirs_next()
        .map(|h| std::path::PathBuf::from(format!("{}/.xiom/registry.json", h)));
    // An EXPLICIT XIOM_HOME always wins; otherwise prefer the canonical file
    // and only fall back to the legacy `~/.xiom/registry.json` when it is
    // the one that actually exists.
    let xiom_home_explicit = std::env::var("XIOM_HOME")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let reg_path = if xiom_home_explicit || new_path.exists() {
        new_path
    } else {
        legacy_path.filter(|p| p.exists()).unwrap_or(new_path)
    };
    let reg_path = reg_path.to_string_lossy().to_string();
    let reg_dir = std::path::Path::new(&reg_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| home.clone());

    let sub_cmd = args.iter()
        .position(|a| a == "registry")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "list".to_string());

    match sub_cmd.as_str() {
        "init" => {
            let initial = serde_json::json!({ "packages": {} });
            if let Ok(json) = serde_json::to_string_pretty(&initial) {
                std::fs::create_dir_all(&reg_dir).ok();
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
                            std::fs::create_dir_all(&reg_dir).ok();
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

fn run_benchmarks(args: &cli::Cli, iterations: u32) {
    // `xiom bench [<dir>]` -- the directory is the positional word after
    // `bench`; `--bench-file` selects a single file.
    let bench_dir = args.iter().position(|a| a == "bench")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "benches".to_string());

    let bench_file = args.value("bench-file")
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
        let xiompath = std::env::current_exe().unwrap_or_else(|_| "xiom".into());

        let compile = std::process::Command::new(&xiompath)
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

    let main_xi = format!(r#"// {name} -- entry point
module {name}

fn main() -> Int {{
    return 0;
}}
"#);
    let main_path = format!("{dir}/src/main.xi");
    if !std::path::Path::new(&main_path).exists() {
        std::fs::write(&main_path, &main_xi).ok();
    }

    let test_xi = format!(r#"// {name} -- tests
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
    eprintln!("  |-- package.xi       <- project manifest");
    eprintln!("  |-- src/main.xi      <- entry point");
    eprintln!("  |-- tests/");
    eprintln!("  |   `-- test_main.xi <- tests");
    eprintln!("  `-- .gitignore");
    eprintln!("  ");
    eprintln!("  Next steps:");
    eprintln!("    cd {dir}");
    eprintln!("    xiom check         <- type-check your project");
    eprintln!("    xiom src/main.xi --run   <- compile and run");
    eprintln!("    xiom test          <- run test suite");
}

/// 9A: xiom doctor -- check all dependencies and report status.
fn run_doctor() {
    println!("XIOM Doctor v{}", env!("CARGO_PKG_VERSION"));
    println!("====================");
    println!();
    println!("  [OK] xiom v{}", env!("CARGO_PKG_VERSION"));
    let clang_ok = std::process::Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    if clang_ok { println!("  [OK] clang/LLVM found"); }
    else { println!("  [!!] clang NOT FOUND - run: xiom install llvm"); }
    match xiom_verify::Z3Runner::find_z3() {
        Some(path) => println!("  [OK] z3 found: {path} (contract verification)"),
        None => println!("  [--] z3 not found (release archives bundle bin/z3)"),
    }
    // CRB-3c: one resolver shared with the installers (canonical
    // `%LOCALAPPDATA%\xiom` / `~/.local/share/xiom`, legacy layouts accepted).
    let home = xiom_graph::paths::xiom_home();
    println!("  [--] XIOM_HOME={}", home.display());
    let lib = home.join("lib").join("xiom");
    if lib.exists() { println!("  [OK] stdlib installed"); }
    else {
        println!("  [!!] stdlib missing - re-run installer");
        if std::env::var("XIOM_HOME").map_or(true, |v| v.trim().is_empty()) {
            let searched: Vec<String> = xiom_graph::paths::xiom_home_candidates()
                .iter().map(|p| p.display().to_string()).collect();
            println!("       searched: {}", searched.join(", "));
        }
    }
    let pkgs = home.join("packages");
    if pkgs.exists() { println!("  [OK] packages directory exists"); }
    else { println!("  [--] No packages (use: xiom pkg install <name>)"); }
}

/// 9B: xiom doc -- generate documentation for XIOM source files.
/// Delegates to the standalone xiom-doc binary, passing through all args
/// after the `doc` subcommand.
fn run_doc(args: &[String]) {
    // Find the position of "doc" or "--doc" in args, pass everything after it
    let doc_pos = args.iter().position(|a| a == "doc" || a == "--doc").unwrap_or(0);
    let doc_args: Vec<&str> = args[doc_pos + 1..].iter().map(|s| s.as_str()).collect();

    // CRB-3c: same installer-aligned home resolver as `xiom doctor`.
    let home = xiom_graph::paths::xiom_home();
    let doc_bin = home.join("bin")
        .join(if cfg!(windows) { "xiom-doc.exe" } else { "xiom-doc" });

    if doc_bin.exists() {
        let mut cmd = std::process::Command::new(&doc_bin);
        cmd.args(&doc_args);
        let status = cmd.status().unwrap_or_else(|e| {
            eprintln!("xiom doc: failed to run xiom-doc: {e}");
            std::process::exit(1);
        });
        std::process::exit(status.code().unwrap_or(1));
    } else {
        eprintln!("xiom doc: xiom-doc binary not found at {}", doc_bin.display());
        eprintln!("  Build it with: cargo build -p xiom-doc --release");
        eprintln!("  Then copy to: {}", doc_bin.display());
        std::process::exit(1);
    }
}
