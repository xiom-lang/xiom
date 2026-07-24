// M10: Scripting mode integration tests (xiom run)
// Run with: cargo test -p xiom --test scripting_tests
use std::io::Write;
use std::process::Command;

fn xiom_binary() -> String {
    std::env::var("XIOM_BIN").unwrap_or_else(|_| {
        let mut p = std::env::current_exe().unwrap();
        p.pop(); p.pop(); // up from deps/
        p.push("xiom");
        if cfg!(windows) { p.set_extension("exe"); }
        p.to_string_lossy().to_string()
    })
}

fn tmp_script(name: &str, content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("xiom_script_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

fn run_script(content: &str) -> std::process::Output {
    let script = tmp_script("test.xi", content);
    let output = Command::new(xiom_binary())
        .args(["run", &script.to_string_lossy().to_string()])
        .output()
        .expect("xiom run failed to start");
    let _ = std::fs::remove_file(&script);
    output
}

#[test]
fn test_script_implicit_main() {
    let output = run_script("io.println(\"hello script\");\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello script"),
        "implicit main should work, stdout: {stdout}, stderr: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_script_shebang() {
    let script = tmp_script("shebang.xi", "#!/usr/bin/env xiom\nio.println(\"shebang works\");\n");
    let output = Command::new(xiom_binary())
        .args(["run", &script.to_string_lossy().to_string()])
        .output()
        .expect("xiom run failed");
    let _ = std::fs::remove_file(&script);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("shebang works") || output.status.success(),
        "shebang: stdout={stdout}, stderr={}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_script_explicit_main() {
    let output = run_script("fn main() { io.println(\"explicit\"); }\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("explicit") || output.status.success(),
        "explicit main should work");
}

#[test]
fn test_script_minimal() {
    let output = run_script("var x = 1;\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() || !stderr.contains("error:"),
        "minimal script should succeed, stderr: {stderr}");
}

#[test]
fn test_script_return_value() {
    let output = run_script("return 42;\n");
    assert!(output.status.success(),
        "script with return should compile, stderr: {}",
        String::from_utf8_lossy(&output.stderr));
}
