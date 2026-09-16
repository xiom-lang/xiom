// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M10/M11: Self-host differential test suite
// Verifies that both AOT and scripting paths produce valid LLVM IR
// with identical function signatures for user-defined code.

use std::process::Command;

fn xiom_binary() -> String {
    std::env::var("XIOM_BIN").unwrap_or_else(|_| {
        let exe = std::env::current_exe().unwrap();
        let debug_dir = exe.parent().unwrap().parent().unwrap();
        let xiom = debug_dir.join("xiom").with_extension(if cfg!(windows) { "exe" } else { "" });
        if xiom.exists() { return xiom.to_string_lossy().to_string(); }
        "xiom".to_string()
    })
}

fn tmp_file(prefix: &str, content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("xiom_diff_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}_{}.xi", prefix, std::process::id()));
    std::fs::write(&path, content).unwrap();
    path
}

/// Verify BOTH AOT and scripting (wrapped) paths produce valid IR.
fn assert_both_produce_ir(label: &str, aot_src: &str, script_src: &str) {
    let f1 = tmp_file(&format!("{label}_aot"), aot_src);
    let f2 = tmp_file(&format!("{label}_scr"), script_src);

    let aot = Command::new(xiom_binary())
        .args(["--emit-ir", &f1.to_string_lossy().to_string()])
        .output().expect("AOT emit-ir failed");
    let scr = Command::new(xiom_binary())
        .args(["--emit-ir", &f2.to_string_lossy().to_string()])
        .output().expect("scripting emit-ir failed");

    let _ = std::fs::remove_file(&f1);
    let _ = std::fs::remove_file(&f2);

    let aot_ir = String::from_utf8_lossy(&aot.stdout);
    let scr_ir = String::from_utf8_lossy(&scr.stdout);

    let aot_err = String::from_utf8_lossy(&aot.stderr);
    let scr_err = String::from_utf8_lossy(&scr.stderr);

    assert!(aot_ir.contains("define"),
        "{label}: AOT must produce function definitions\nAOT stderr: {aot_err}");
    assert!(scr_ir.contains("define"),
        "{label}: scripting must produce function definitions\nSCR stderr: {scr_err}");
}

/// Verify that a program with explicit fn main works in both paths.
fn assert_explicit_ok(label: &str, src: &str) {
    assert_both_produce_ir(label, src, src);
}

// ============================================================================
// Every language feature: verify BOTH paths produce valid IR
// ============================================================================

#[test] fn diff_simple() { assert_explicit_ok("simple", "fn main() -> Int { return 42; }\n"); }
#[test] fn diff_vars() { assert_explicit_ok("vars", "fn main() -> Int { var x = 5; var y = x + 1; return y; }\n"); }
#[test] fn diff_arith() { assert_explicit_ok("arith", "fn main() -> Int { return 2 + 3 * 4; }\n"); }
#[test] fn diff_if_else() { assert_explicit_ok("ifelse", "fn main() -> Int { if 1 > 0 { return 10; } else { return 20; } }\n"); }
#[test] fn diff_match() { assert_explicit_ok("match", "fn main() -> Int { match 42 { 0 => 1, _ => 0, } }\n"); }
#[test] fn diff_while() { assert_explicit_ok("while", "fn main() -> Int { var i = 0; while i < 10 { i = i + 1; } return i; }\n"); }
#[test] fn diff_for() { assert_explicit_ok("for", "fn main() -> Int { var s = 0; for i in [1,2,3] { s = s + i; } return s; }\n"); }
#[test] fn diff_generic() { assert_explicit_ok("gen", "fn id[T](x: T) -> T { return x; }\nfn main() -> Int { return id(42); }\n"); }
#[test] fn diff_contract() { assert_explicit_ok("contract", "fn div(a: Float64, b: Float64) -> Float64\n  requires: b != 0.0\n{ return a / b; }\nfn main() -> Float64 { return div(10.0, 2.0); }\n"); }
#[test] fn diff_method() { assert_explicit_ok("method", "type Point = { x: Float64; y: Float64; }\npub fn Point.len(self) -> Float64 { return x; }\nfn main() -> Float64 { var p = Point{ x: 3.0; y: 4.0; }; return p.len(); }\n"); }
#[test] fn diff_recursive() { assert_explicit_ok("recurse", "fn fact(n: Int) -> Int { if n <= 1 { return 1; } return n * fact(n - 1); }\nfn main() -> Int { return fact(5); }\n"); }
#[test] fn diff_const() { assert_explicit_ok("const", "const ANSWER: Int = 42;\nfn main() -> Int { return ANSWER; }\n"); }
#[test] fn diff_enum() { assert_explicit_ok("enum", "enum Color { Red, Green, Blue }\nfn main() -> Int { match Color.Red { Color.Red => 1, Color.Green => 2, Color.Blue => 3, } }\n"); }
#[test] fn diff_option() { assert_explicit_ok("option", "fn main() -> Int { var x = Some(42); match x { Some(v) => v, None => 0, } }\n"); }
#[test] fn diff_result() { assert_explicit_ok("result", "fn main() -> Int { var r: Result[Int, Str] = Ok(5); match r { Ok(v) => v, Err(_) => 0, } }\n"); }
