// XIOM -- Stdlib Compilation Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Verifies that every stdlib .xi module compiles to IR with the current compiler.

use std::process::Command;
use std::path::Path;
use std::io::Write;
use std::fs;

fn xiom_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiom.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiom.exe");
    }
    path.to_str().unwrap().to_string()
}

fn project_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .to_path_buf()
    }).as_path()
}

/// All stdlib .xi files to compile (module name -> relative path from project root)
fn stdlib_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("core",        "stdlib/xiom/core.xi"),
        ("array",       "stdlib/xiom/array.xi"),
        ("string",      "stdlib/xiom/string.xi"),
        ("collections", "stdlib/xiom/collections.xi"),
        ("io",          "stdlib/xiom/io.xi"),
        ("fmt",         "stdlib/xiom/fmt.xi"),
        ("iter",        "stdlib/xiom/iter.xi"),
        ("math",        "stdlib/xiom/math.xi"),
        ("num",         "stdlib/xiom/num.xi"),
        ("cmp",         "stdlib/xiom/cmp.xi"),
        ("hash",        "stdlib/xiom/hash.xi"),
        ("mem",         "stdlib/xiom/mem.xi"),
        ("ptr",         "stdlib/xiom/ptr.xi"),
        ("char",        "stdlib/xiom/char.xi"),
        ("path",        "stdlib/xiom/path.xi"),
        ("convert",     "stdlib/xiom/convert.xi"),
        ("ffi",         "stdlib/xiom/ffi.xi"),
        ("sync",        "stdlib/xiom/sync.xi"),
        ("thread",      "stdlib/xiom/thread.xi"),
        ("async",       "stdlib/xiom/async.xi"),
        ("net",         "stdlib/xiom/net.xi"),
        ("os",          "stdlib/xiom/os.xi"),
        ("time",        "stdlib/xiom/time.xi"),
        ("test",        "stdlib/xiom/test.xi"),
        ("bench",       "stdlib/xiom/bench.xi"),
        ("log",         "stdlib/xiom/log.xi"),
        ("serialize",   "stdlib/xiom/serialize.xi"),
        ("crypto",      "stdlib/xiom/crypto.xi"),
        ("regex",       "stdlib/xiom/regex.xi"),
        ("rand",        "stdlib/xiom/rand.xi"),
        ("encoding",    "stdlib/xiom/encoding.xi"),
        ("compress",    "stdlib/xiom/compress.xi"),
        ("contracts",   "stdlib/xiom/contracts.xi"),
        ("reflect",     "stdlib/xiom/reflect.xi"),
        ("cell",        "stdlib/xiom/cell.xi"),
        ("rc",          "stdlib/xiom/rc.xi"),
        ("alloc",       "stdlib/xiom/alloc.xi"),
        ("env",         "stdlib/xiom/env.xi"),
        ("error",       "stdlib/xiom/error.xi"),
        // Folder modules (2026-08-07 refactor) -- resolved via catalog strategy a:
        // `use xiom.foo.bar` -> stdlib/xiom/foo/bar.xi
        ("collect.tree",   "stdlib/xiom/collect/tree.xi"),
        ("collect.heap",   "stdlib/xiom/collect/heap.xi"),
        ("collect.cache",  "stdlib/xiom/collect/cache.xi"),
        ("collect.hash",   "stdlib/xiom/collect/hash.xi"),
        ("collect.queue",  "stdlib/xiom/collect/queue.xi"),
        ("collect.graph",  "stdlib/xiom/collect/graph.xi"),
        ("hash.city",      "stdlib/xiom/hash/city.xi"),
        ("hash.xxhash",    "stdlib/xiom/hash/xxhash.xi"),
        ("hash.murmur",    "stdlib/xiom/hash/murmur.xi"),
        ("hash.jenkins",   "stdlib/xiom/hash/jenkins.xi"),
        ("hash.crc",       "stdlib/xiom/hash/crc.xi"),
        ("text.similarity","stdlib/xiom/text/similarity.xi"),
        ("rand.mt19937",   "stdlib/xiom/rand/mt19937.xi"),
        ("rand.pcg",       "stdlib/xiom/rand/pcg.xi"),
        ("rand.chacha",    "stdlib/xiom/rand/chacha.xi"),
        ("net.url",        "stdlib/xiom/net/url.xi"),
        ("net.dns",        "stdlib/xiom/net/dns.xi"),
        ("net.proto",      "stdlib/xiom/net/proto.xi"),
        ("os.fs",          "stdlib/xiom/os/fs.xi"),
        ("os.proc",        "stdlib/xiom/os/proc.xi"),
        ("os.term",        "stdlib/xiom/os/term.xi"),
        ("num.convert",    "stdlib/xiom/num/convert.xi"),
        ("format.number",  "stdlib/xiom/format/number.xi"),
        ("format.dump",    "stdlib/xiom/format/dump.xi"),
    ]
}

#[test]
fn stdlib_all_modules_compile_to_ir() {
    // R31: a missing stdlib checkout is a loud SKIP locally and a hard FAIL
    // under XIOM_REQUIRE_STDLIB=1 (CI) -- never a silent green.
    if xiom_graph::paths::stdlib_or_skip().is_none() {
        return;
    }
    // Compile ALL stdlib modules TOGETHER in a single compilation unit
    // so cross-module references (e.g. core.xi::from_cstring used by
    // string.xi) are resolved. Isolated per-file compilation was failing
    // because modules depend on each other -- not because of checker bugs.
    let modules = stdlib_modules();
    let total = modules.len();
    let project_dir = project_root().display().to_string();

    // Build a synthetic program that imports all stdlib modules.
    // We use `use xiom.XXX;` to bring each module into scope, then
    // a minimal main() to ensure the linker can generate code.
    let mut program = String::new();
    for (name, _path) in &modules {
        program.push_str(&format!("use xiom.{};\n", name));
    }
    program.push_str("fn main() -> Int { return 0; }\n");

    // Write to a temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_file = tmp_dir.join("xiom_stdlib_full_test.xi");
    let mut f = fs::File::create(&tmp_file).expect("create temp file");
    f.write_all(program.as_bytes()).expect("write temp file");
    let tmp_path = tmp_file.to_str().unwrap().to_string();

    // Compile with --emit-ir (checker pass is required)
    let output = Command::new(xiom_path())
        .args(["--emit-ir", &tmp_path])
        .current_dir(&project_dir)
        .output()
        .expect("failed to execute xiom");

    let _ = fs::remove_file(&tmp_file);

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "stdlib full compilation failed ({} modules together):\n{}",
        total, stderr
    );
}

// =====================================================================
// Individual per-module tests (stdlib validation with granularity).
// Each module is compiled with `use xiom.core;` for cross-module deps.
// =====================================================================

fn compile_module(module: &str) -> bool {
    if xiom_graph::paths::stdlib_or_skip().is_none() {
        return true; // skipped loudly above
    }
    let project_dir = project_root().display().to_string();
    let program = format!("use xiom.core;\nuse xiom.{};\nfn main() -> Int {{ return 0; }}\n", module);
    let tmp_file = std::env::temp_dir().join(format!("xiom_stdlib_{}.xi", module));
    fs::write(&tmp_file, program).expect("write temp file");
    let tmp_path = tmp_file.to_str().unwrap().to_string();
    let output = Command::new(xiom_path())
        .args(["--emit-ir", &tmp_path])
        .current_dir(&project_dir)
        .output()
        .expect("failed to execute xiom");
    let _ = fs::remove_file(&tmp_file);
    output.status.success()
}

macro_rules! stdlib_test {
    ($name:ident, $module:literal) => {
        #[test]
        fn $name() {
            assert!(compile_module($module), "stdlib module '{}' must compile", $module);
        }
    };
}

stdlib_test!(stdlib_core,        "core");
stdlib_test!(stdlib_array,       "array");
stdlib_test!(stdlib_string,      "string");
stdlib_test!(stdlib_collections, "collections");
stdlib_test!(stdlib_io,          "io");
stdlib_test!(stdlib_fmt,         "fmt");
stdlib_test!(stdlib_iter,        "iter");
stdlib_test!(stdlib_math,        "math");
stdlib_test!(stdlib_num,         "num");
stdlib_test!(stdlib_cmp,         "cmp");
stdlib_test!(stdlib_hash,        "hash");
stdlib_test!(stdlib_mem,         "mem");
stdlib_test!(stdlib_ptr,         "ptr");
stdlib_test!(stdlib_char,        "char");
stdlib_test!(stdlib_path,        "path");
stdlib_test!(stdlib_convert,     "convert");
stdlib_test!(stdlib_ffi,         "ffi");
stdlib_test!(stdlib_sync,        "sync");
stdlib_test!(stdlib_thread,      "thread");
stdlib_test!(stdlib_async,       "async");
stdlib_test!(stdlib_net,         "net");
stdlib_test!(stdlib_os,          "os");
stdlib_test!(stdlib_time,        "time");
stdlib_test!(stdlib_test,        "test");
stdlib_test!(stdlib_bench,       "bench");
stdlib_test!(stdlib_log,         "log");
stdlib_test!(stdlib_serialize,   "serialize");
stdlib_test!(stdlib_crypto,      "crypto");
stdlib_test!(stdlib_regex,       "regex");
stdlib_test!(stdlib_rand,        "rand");
stdlib_test!(stdlib_encoding,    "encoding");
stdlib_test!(stdlib_compress,    "compress");
stdlib_test!(stdlib_contracts,   "contracts");
stdlib_test!(stdlib_reflect,     "reflect");
stdlib_test!(stdlib_cell,        "cell");
stdlib_test!(stdlib_rc,          "rc");
stdlib_test!(stdlib_alloc,       "alloc");
stdlib_test!(stdlib_env,         "env");
stdlib_test!(stdlib_error,       "error");
