# XIOM fuzz targets

Coverage-guided fuzzing for the compiler front half, replacing the toy LCG
harnesses (audit finding: "toy fuzzers; no coverage/ASAN/fuzz CI gates").

This directory is a STANDALONE cargo workspace (it pulls `libfuzzer-sys`,
which must not enter `cargo check --workspace --all-targets`), so `cargo
fuzz` discovers it from the repo root while the root workspace stays clean.

## Targets

| Target     | Surface                                             |
|------------|-----------------------------------------------------|
| `lexer`    | arbitrary bytes as lossy UTF-8 -> `Lexer::tokenize` |
| `parser`   | tokenize + `parse_program` (depth guard, recovery)  |
| `ctfe`     | parse -> register fns -> evaluate (fuel/depth caps) |
| `pipeline` | parse -> `IrEmitter::compile_program` (codegen)     |

Every target asserts the audited invariant: adversarial input must return
Ok/Err, never panic and never hang.

## CI

`.github/workflows/ci.yml` job `fuzz-smoke` builds all targets on nightly and
runs each for 30s with ASAN (Linux links the sanitizer statically); crashes
upload `fuzz/artifacts/<target>/`. The `sanitizer-smoke` job builds the
compiler and runs a `--sanitize=address` executable.

## Local usage

```sh
cargo +nightly fuzz run lexer -- -max_total_time=60
cargo +nightly fuzz run pipeline -- -max_total_time=60
# Reproduce a crash:
cargo +nightly fuzz run lexer fuzz/artifacts/lexer/crash-<hash>
```

Windows/MSVC note: Rust's ASAN needs the `clang_rt.asan_dynamic-x86_64.dll`
matching the toolchain's LLVM version. If the local LLVM install does not
match, `cargo +nightly fuzz run` fails to start the target
(`STATUS_DLL_NOT_FOUND` / `STATUS_ENTRYPOINT_NOT_FOUND`); run the targets on
Linux/WSL or in CI for sanitizer coverage. `--sanitizer=none` is not
supported by libfuzzer-sys on MSVC (unresolved sancov symbols).

## Seeds

`corpus/<target>/` holds small seeds checked into git. New findings land in
`fuzz/artifacts/<target>/` (ignored) and should be minimized
(`cargo +nightly fuzz tmin <target> <artifact>`) and added to the corpus with
a regression note.
