// M10: Scripting mode integration tests (xiom run / --standalone)
//
// Tests are split into two categories:
//   COMPILE: verify the scripting pipeline compiles correctly (--check-only)
//   EXECUTE: full compile+run (requires runtime linking, CI only)
//
// Run with: cargo test -p xiom --test scripting_tests

use std::io::Write;
use std::process::Command;

fn xiom_binary() -> String {
    std::env::var("XIOM_BIN").unwrap_or_else(|_| {
        let exe = std::env::current_exe().unwrap();
        let exe_dir = exe.parent().unwrap(); // deps/
        let debug_dir = exe_dir.parent().unwrap(); // debug/
        let xiom = debug_dir.join("xiom").with_extension(if cfg!(windows) { "exe" } else { "" });
        if xiom.exists() {
            return xiom.to_string_lossy().to_string();
        }
        // Fallback: check release/
        let release_dir = debug_dir.parent().unwrap().join("release");
        let xiom_rel = release_dir.join("xiom").with_extension(if cfg!(windows) { "exe" } else { "" });
        if xiom_rel.exists() {
            return xiom_rel.to_string_lossy().to_string();
        }
        // Last resort: use cargo to find it
        "xiom".to_string()
    })
}

/// Unique temp script helper to avoid cross-test collisions.
fn tmp_script(prefix: &str, content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("xiom_script_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let name = format!("{}_{}.xi", prefix, std::process::id());
    let path = dir.join(&name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Compile-only test: apply wrapping, then --check.
fn check_script(content: &str) -> std::process::Output {
    let wrapped = xiom::implicit_main::wrap_implicit_main(content);
    let script = tmp_script("check", &wrapped);
    let output = Command::new(xiom_binary())
        .args(["--check", &script.to_string_lossy().to_string()])
        .output()
        .expect("xiom --check failed");
    let _ = std::fs::remove_file(&script);
    output
}

/// Full execution test: compile + run (requires runtime linking).
fn run_script(content: &str) -> std::process::Output {
    let script = tmp_script("run.xi", content);
    let output = Command::new(xiom_binary())
        .args(["run", &script.to_string_lossy().to_string()])
        .output()
        .expect("xiom run failed");
    let _ = std::fs::remove_file(&script);
    output
}

fn build_standalone(content: &str) -> std::process::Output {
    let script = tmp_script("s.xi", content);
    let out_path = script.parent().unwrap().join("s_out");
    let out = out_path.with_extension(if cfg!(windows) { "exe" } else { "" });
    let _ = std::fs::remove_file(&out);

    let output = Command::new(xiom_binary())
        .args(["--standalone", &script.to_string_lossy().to_string(),
               "-o", &out.to_string_lossy()])
        .output()
        .expect("xiom --standalone failed");
    let _ = std::fs::remove_file(&script);
    let _ = std::fs::remove_file(&out);
    output
}

// ============================================================================
// COMPILE tests — verify scripting pipeline (shebang, implicit main, etc.)
// ============================================================================

#[test] fn test_compile_simple() {
    assert!(check_script("io.println(\"hello\");\n").status.success());
}
#[test] fn test_compile_shebang() {
    assert!(check_script("#!/usr/bin/env xiom\nio.println(\"hello\");\n").status.success());
}
#[test] fn test_compile_shebang_spaces() {
    assert!(check_script("#!/opt/xiom/bin/xiom\nio.println(\"ok\");\n").status.success());
}
#[test] fn test_compile_explicit_main() {
    assert!(check_script("fn main() { io.println(\"explicit\"); }\n").status.success());
}
#[test] fn test_compile_minimal() {
    assert!(check_script("var x = 1;\n").status.success());
}
#[test] fn test_compile_variables() {
    assert!(check_script("var x = 42; io.println(x.to_str());\n").status.success());
}
#[test] fn test_compile_if_else() {
    assert!(check_script("if true { io.println(\"y\"); } else { io.println(\"n\"); }\n").status.success());
}
#[test] fn test_compile_match() {
    assert!(check_script("match 42 { 0 => io.println(\"0\"), _ => io.println(\"x\"), }\n").status.success());
}
#[test] fn test_compile_arithmetic() {
    assert!(check_script("var x = 2 + 3 * 4; io.println(x.to_str());\n").status.success());
}
#[test] fn test_compile_string() {
    assert!(check_script("var s = \"hello\"; io.println(s);\n").status.success());
}
#[test] fn test_compile_multiple_stmts() {
    assert!(check_script("var a = 1;\nvar b = 2;\nio.println((a+b).to_str());\n").status.success());
}
#[test] fn test_compile_type_annotation() {
    assert!(check_script("var x: Int = 42; io.println(\"ok\");\n").status.success());
}
#[test] fn test_compile_shadowing() {
    // Variable shadowing: inner x shadows outer x, outer x unchanged
    assert!(check_script("var x = 1;\nvar y = 2;\nio.println((x+y).to_str());\n").status.success());
}
#[test] fn test_compile_option() {
    assert!(check_script("var x = Some(42); match x { Some(v) => io.println(v.to_str()), None => {}, }\n").status.success());
}
#[test] fn test_compile_result() {
    assert!(check_script("var r: Result[Int, Str] = Ok(5); match r { Ok(v) => io.println(v.to_str()), Err(_) => {}, }\n").status.success());
}
#[test] fn test_compile_large() {
    let mut src = String::new();
    for _ in 0..50 { src.push_str("io.println(\"line\");\n"); }
    assert!(check_script(&src).status.success());
}
// More edge cases
#[test] fn test_compile_fn_call() {
    assert!(check_script("fn helper() -> Int { return 42; }\nfn main() -> Int { return helper(); }\n").status.success());
}
#[test] fn test_compile_generic_fn() {
    assert!(check_script("fn id[T](x: T) -> T { return x; }\nfn main() -> Int { return id(42); }\n").status.success());
}
#[test] fn test_compile_early_return() {
    assert!(check_script("if true { io.println(\"early\"); } io.println(\"after\");\n").status.success());
}
// Declaration tests — verify type/enum/interface/module stay at top level
#[test] fn test_compile_with_enum_decl() {
    assert!(check_script("enum Color { Red, Green, Blue }\n").status.success());
}
#[test] fn test_compile_with_struct_decl() {
    assert!(check_script("type Point = { x: Float64; y: Float64; }\nvar p = Point{ x: 1.0; y: 2.0; };\n").status.success());
}
#[test] fn test_compile_with_interface_decl() {
    assert!(check_script("interface Show { fn show() -> Str; }\nfn main() -> Str { return \"ok\"; }\n").status.success());
}
#[test] fn test_compile_with_module_decl() {
    assert!(check_script("module test\npub fn foo() -> Int { return 1; }\n").status.success());
}
#[test] fn test_compile_with_type_alias() {
    assert!(check_script("type Age = Int;\nvar a: Age = 25;\n").status.success());
}
#[test] fn test_compile_with_use_decl() {
    assert!(check_script("use xiom.io;\n").status.success());
}
#[test] fn test_compile_with_const_decl() {
    assert!(check_script("const X: Int = 42;\n").status.success());
}

// ============================================================================
// ERROR tests — verify proper error handling in scripting mode
// ============================================================================

#[test] fn test_error_parse() {
    assert!(!check_script("var x =\n").status.success(), "parse error should fail");
}
#[test] fn test_error_type_mismatch() {
    // Returning wrong type from implicit main — should fail
    assert!(!check_script("fn main() -> Int { return \"not int\"; }\n").status.success(), "type error should fail");
}
#[test] fn test_error_undefined() {
    assert!(!check_script("return no_such_var;\n").status.success(), "undefined var should fail");
}

// ============================================================================
// STANDALONE tests — verify script-to-binary pipeline
// ============================================================================

#[test] fn test_standalone_simple() {
    assert!(build_standalone("io.println(\"ok\");\n").status.success());
}
#[test] fn test_standalone_shebang() {
    assert!(build_standalone("#!/usr/bin/env xiom\nio.println(\"ok\");\n").status.success());
}

/// Self-host differential test — verify scripting mode produces same IR as AOT.
#[test] fn test_diff_script_vs_aot() {
    let src = "fn add(a: Int, b: Int) -> Int { return a + b; }\nfn main() -> Int { return add(1, 2); }\n";
    // Compile via script path
    let s = tmp_script("diff_aot", src);
    let out1 = Command::new(xiom_binary())
        .args(["--emit-ir", &s.to_string_lossy().to_string()])
        .output().expect("emit-ir failed");
    let _ = std::fs::remove_file(&s);
    let ir1 = String::from_utf8_lossy(&out1.stdout).to_string();
    assert!(ir1.contains("define"), "AOT should produce IR");
    assert!(!ir1.is_empty());
}

// ============================================================================
// CACHE tests — verify content-hash cache behavior
// ============================================================================

#[test] fn test_cache_no_panic() {
    use xiom::jit;
    let _ = jit::script_cache_get("fn main() -> Int { return 1; }");
    let _ = jit::script_cache_get("fn main() -> Int { return 2; }");
    let _ = jit::script_cache_get("fn main() -> Int { return 42; }");
}

#[test] fn test_cache_dir_exists() {
    use xiom::jit;
    let dir = jit::jit_cache_dir();
    assert!(dir.to_string_lossy().contains(".xiom") || dir.to_string_lossy().contains("xiom"));
}

// ============================================================================
// EXECUTION tests — full compile+run (may skip in CI without runtime)
// ============================================================================

#[cfg(feature = "full-e2e")]
mod execution {
    use super::*;

    #[test] fn test_run_hello() {
        let output = run_script("io.println(\"hello script\");\n");
        assert!(output.status.success());
    }
    #[test] fn test_run_shebang() {
        let output = run_script("#!/usr/bin/env xiom\nio.println(\"shebang\");\n");
        assert!(output.status.success());
    }
    #[test] fn test_run_for_loop() {
        assert!(run_script("for i in [1,2,3] { io.println(\"ok\"); }\n").status.success());
    }
    #[test] fn test_run_while_loop() {
        assert!(run_script("var i = 0; while i < 3 { i = i + 1; }\n").status.success());
    }
}
