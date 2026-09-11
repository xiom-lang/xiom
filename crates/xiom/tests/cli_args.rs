// Driver argument-resolution regression (Stage 5 driver item): the manual
// parser used to drop files literally named wasm/arm/riscv as if they were
// positional target sugar.

use xiom::resolve_source_files;

#[test]
fn target_named_source_files_are_kept_but_bare_sugar_is_skipped() {
    let dir = std::env::temp_dir().join("xiom_cli_args_test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("wasm"), "fn main() -> Int { return 0; }").unwrap();
    std::fs::write(dir.join("arm"), "fn main() -> Int { return 0; }").unwrap();

    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    // Real files named like targets are SOURCES, not sugar.
    let kept = resolve_source_files(&["xiom".to_string(), "wasm".to_string(), "arm".to_string()]);
    assert_eq!(kept, vec!["wasm".to_string(), "arm".to_string()]);

    // A bare `wasm` with no such file stays skipped (positional target).
    std::fs::remove_file(dir.join("wasm")).unwrap();
    let skipped = resolve_source_files(&["xiom".to_string(), "wasm".to_string(), "src.xi".to_string()]);
    assert_eq!(skipped, vec!["src.xi".to_string()]);

    std::env::set_current_dir(old).unwrap();
}
