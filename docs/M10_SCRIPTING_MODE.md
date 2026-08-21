# M10 -- XIOM Scripting / JIT Mode

**Date:** 2026-07-24 | **Status:** Design Phase | **Target:** v0.50.0 "Scripting Edition"
**Dependency:** M1-M9 complete. Requires `xiom run` CLI + JIT backend.
**Implementation standard:** FULL PRODUCTION. No MVP, no shortcuts. Every feature ships
with complete error handling, documentation, TDD test suite, and self-host differential verification.

## 1. Motivation

XIOM is a compiled, statically-typed systems language that produces native binaries via LLVM.
Users and AI agents want the same "write and run" experience as Python scripts -- no compile/link
cycle, no build artifacts, instant feedback.

Because XIOM's compiler pipeline (lex -> parse -> check -> IR) is already modular, we can add a JIT
execution path that shares the same AST, type checker, and contract system as AOT compilation.
This means **scripts are NOT a different language** -- they are the same XIOM code routed through
a different execution backend. A script can be "graduated" into a production binary with zero code
changes by switching from `xiom run` to `xiom build`.

### Key use cases

| User | Workflow |
|------|----------|
| Developer | Prototype in JIT mode, graduate to AOT binary when performance matters |
| AI Agent (MCP) | Sub-10ms feedback loop for code generation and testing |
| DevOps/Scripting | `#!/usr/bin/env xiom` shebang scripts replacing Bash/Python for system automation |
| Self-hosting bootstrap | JIT the self-host compiler during development; AOT for releases |

## 2. Architecture

```
                       SAME XIOM AST / CONTRACTS / CHECKER
                                    |
         +--------------------------+--------------------------+
         v                                                      v
+---------------------+                            +---------------------+
|  SCRIPTING / JIT MODE                             |  PRODUCTION / AOT MODE
|                                                    |
|  xiom run script.xi                                |  xiom build script.xi -o bin
|  xiom run -e "print(42)"                           |  xiom build --release
|  #!/usr/bin/env xiom (shebang)                     |
|                                                    |
|  Pipeline:                                         |  Pipeline:
|  1. Lex -> Parse -> Check                            |  1. Lex -> Parse -> Check
|  2. Wrap top-level stmts in implicit main()        |  2. Full LLVM -O3 passes
|  3. JIT via inkwell (or Cranelift)                 |  3. Emit native binary to disk
|  4. Execute immediately in-process                 |  4. No runtime dependency
|                                                    |
|  Characteristics:                                  |  Characteristics:
|  - O0 / fast compilation                           |  - O3 / full optimization
|  - Zero disk artifacts                             |  - Static binary
|  - 10-50ms startup for small scripts               |  - C/Rust-level performance
|  - Interactive REPL (later)                        |  - Production deployment
`---------------------+                            `---------------------+
```

### Self-hosting consideration

When XIOM is self-hosted (compiler written in XIOM, compiled by XIOM), the JIT mode MUST
be able to JIT-compile the compiler itself. This means:

1. **The JIT backend must support the full XIOM language** -- no subset, no shortcuts.
   Self-hosting the compiler exercises every language feature (generics, contracts, async,
   unsafe, FFI).

2. **Inkwell (LLVM-C API) is the recommended JIT backend** because:
   - It wraps the same LLVM that AOT mode uses -- same IR, same lowering
   - LLVM ORC JITv2 supports lazy compilation (compile functions on first call)
   - Cross-platform (Windows/Linux/macOS)
   - Already a Rust crate with active maintenance

3. **Cranelift is the fallback for ultra-fast startup** -- if LLVM JIT initialization
   latency (100-200ms) is unacceptable for tiny scripts, Cranelift can serve as a
   lighter JIT for the O0 tier. The trade-off is that Cranelift may not support every
   LLVM feature the self-host compiler needs.

4. **The JIT path must be tested with the full selfhost compiler** -- differential testing:
   JIT the selfhost compiler, use it to compile a test program, compare the output with
   the AOT-compiled selfhost compiler. Results must be identical.

## 3. Feature Breakdown

### 3.1 `xiom run` -- JIT Execution (Phase 1, 3d)

```bash
# Execute a .xi file as a script
xiom run script.xi
xiom run script.xi -- arg1 arg2     # pass arguments

# Execute inline code
xiom run -e "io.println('hello')"
xiom run -e "let x = 42; io.println(x.to_str())"

# Execute from stdin (pipe)
echo 'io.println("from pipe")' | xiom run -
cat script.xi | xiom run -
```

**Implementation:**
- `xiom run <file>`: parse, check, JIT-compile, execute `main()` (or implicit main)
- `xiom run -e <code>`: parse inline string, wrap in implicit main, JIT, execute
- `xiom run -`: read from stdin, same as file mode
- Exit code = return value of `main()` (or 0 for void main)

### 3.2 Implicit Main Wrapping (Phase 1, 1d)

Top-level statements are wrapped in an implicit `fn main()`:

```xiom
// User writes:
let files = io.list_dir("./data");
for f in files {
    io.println(f);
}

// Compiler treats as:
fn main() {
    let files = io.list_dir("./data");
    for f in files {
        io.println(f);
    }
}
```

**Rules:**
- If the file already has an explicit `fn main()`, do NOT wrap
- Top-level `module` declarations stay at top level
- Top-level `type`, `enum`, `interface`, `const` declarations stay at top level
- Top-level `use` imports stay at top level
- Everything else (expressions, let/var, control flow, function calls) goes into implicit main
- Implicit main returns `Int` if the last expression is `Int`, otherwise void

### 3.3 Shebang Support (Phase 1, 0.5d)

The lexer treats `#!` on line 1 as a comment:

```xiom
#!/usr/bin/env xiom
use xiom.io;
io.println("Hello from executable script!");
```

```bash
chmod +x script.xi
./script.xi
```

**Implementation:** In the lexer, if the first two characters of the file are `#!`, skip the
entire first line. No other changes needed.

### 3.4 `xiom build --standalone` -- Script-to-Binary (Phase 2, 2d)

"Graduate" a script into a production binary:

```bash
xiom build script.xi --standalone -o tool.exe
```

**What it does:**
1. If the script uses implicit main, extract it into an explicit `fn main() -> Int`
2. Wrap top-level code in a proper module structure if needed
3. Run full LLVM -O3 optimization
4. Emit a static native binary -- zero dependencies beyond libc

```bash
# Optional: also generate a standardized project structure
xiom build script.xi --standalone --scaffold
# Creates:
#   tool/
#     package.xi
#     src/main.xi     (canonicalized from script)
```

### 3.5 REPL (Phase 3, 2d, deferred)

Interactive Read-Eval-Print Loop:

```bash
xiom repl
xiom> let x = 42
xiom> x * 2
84
xiom> :type x
Int
xiom> :quit
```

Each line is JIT-compiled and executed immediately. The REPL maintains variable state
across lines (via a persistent JIT session).

### 3.6 Hot Reload Integration (Phase 3, 2d, deferred)

`xiom run --watch script.xi` re-JITs on file change. Shares infrastructure with the
existing hot-reload system in `xiom-codegen`.

## 4. Technical Implementation

### 4.1 JIT Backend (inkwell crate)

Add to `Cargo.toml`:
```toml
[dependencies]
inkwell = { version = "0.5", features = ["llvm17-0", "target-x86", "target-arm", "target-riscv"] }
```

New module: `crates/xiom-codegen/src/jit.rs`

```rust
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::OptimizationLevel;

pub struct JitCompiler {
    context: Context,
    engine: ExecutionEngine,
}

impl JitCompiler {
    pub fn new() -> Self { /* create LLVM context + ORC JIT engine */ }
    pub fn compile_module(&self, llvm_ir: &str) -> Result<(), String> { /* add module to JIT */ }
    pub fn run_main(&self, args: Vec<String>) -> Result<i32, String> { /* lookup main, call it */ }
    pub fn run_function<T>(&self, name: &str, args: ...) -> Result<T, String> { /* generic fn call */ }
}
```

### 4.2 Implicit Main Transformer

New module: `crates/xiom/src/implicit_main.rs`

```rust
pub fn wrap_implicit_main(program: &mut Program) -> bool {
    // Walk top-level items. Collect non-declaration statements into implicit main.
    // Insert `fn main() { ... }` at the end of program.items.
    // Return true if wrapping occurred.
}
```

This runs after parsing, before checking. The checker sees a normal `fn main()`.

### 4.3 Shebang Lexer Change

In `crates/xiom-lexer/src/lib.rs`, in the `Lexer::new()` or `tokenize()` entry point:

```rust
// Skip shebang line if present
if source.starts_with("#!") {
    if let Some(newline) = source.find('\n') {
        source = &source[newline + 1..];
    }
}
```

### 4.4 CLI Integration

New subcommand in `crates/xiom/src/main.rs`:

```rust
Subcommand::Run { file: Option<String>, expr: Option<String>, args: Vec<String> } => {
    if let Some(expr) = expr {
        run_inline(&expr, args);
    } else if let Some(file) = file {
        if file == "-" {
            run_stdin(args);
        } else {
            run_file(&file, args);
        }
    }
}
```

## 5. Performance Targets

| Scenario | Target | Measurement |
|----------|--------|-------------|
| `xiom run hello.xi` (JIT startup) | <50ms | Wall clock, cold start |
| `xiom run hello.xi` (warm, same process) | <5ms | Incremental re-JIT |
| `xiom run -e "1+1"` | <20ms | Minimal JIT path |
| `xiom run selfhost/lexer.xi` (large file) | <500ms | Full compiler JIT |
| `xiom build --standalone script.xi` | Same as current AOT | No regression |

## 6. Test Plan (TDD required)

### 6.1 Parser tests (shebang)
```
test_shebang_skipped -- #!/usr/bin/env xiom followed by valid code parses correctly
test_shebang_preserves_line_numbers -- error messages report correct lines after shebang
test_no_shebang_normal -- normal file without shebang still works
```

### 6.2 Implicit main tests
```
test_implicit_main_simple -- top-level print -> wrapped in main()
test_implicit_main_with_types -- type/enum/const decls stay at top, code in main()
test_implicit_main_explicit_main_exists -- file with fn main() is NOT wrapped
test_implicit_main_module_first -- module declaration stays at top
test_implicit_main_return_value -- last expression Int -> main returns Int
test_implicit_main_void -- no return expression -> main returns void
```

### 6.3 JIT execution tests
```
test_jit_simple_int -- fn main() -> Int { return 42; } JIT-executes, exit code 42
test_jit_print -- fn main() { io.println("hello"); } JIT-executes, stdout captured
test_jit_contracts -- fn with requires catches violation at JIT runtime
test_jit_generics -- generic fn JIT-executes correctly
test_jit_stdlib -- uses Vec, Option, Result via JIT
```

### 6.4 Script-to-binary tests
```
test_standalone_build -- implicit main script -> xiom build --standalone -> runs identically
test_standalone_perf -- standalone binary's IR matches AOT IR (differential)
```

### 6.5 Self-hosting JIT test (critical)
```
test_jit_selfhost_lexer -- JIT the selfhost lexer, use it to lex a test file
test_jit_selfhost_parser -- JIT the selfhost parser, use it to parse a test file
test_jit_selfhost_full -- JIT the full selfhost compiler, compile a test.xi -> run -> verify output
```

## 7. Phase Schedule (All Production-Grade)

| Phase | Items | Effort | Dependencies |
|-------|-------|--------|-------------|
| **M10.1** Foundation | Shebang lexer + implicit main + `xiom run` CLI (inkwell JIT). Full error handling on JIT failures, stdin/e/file modes, argument passing. | 5d | M9 done |
| **M10.2** Standalone | `xiom build --standalone` + scaffold. Differential testing: standalone binary IR must match AOT IR byte-for-byte. | 3d | M10.1 |
| **M10.3** Self-host JIT | JIT the full selfhost compiler. Full E2E differential testing. JIT and AOT must produce identical stdout/stderr/exit code on 100% of the test suite. | 4d | M10.1 + selfhost functional |
| **M10.4** REPL | Interactive `xiom repl` with persistent JIT session, type inspection (`:type`), variable shadowing across lines, history, readline support. | 3d | M10.1 |
| **M10.5** Hot reload JIT | `xiom run --watch` file watcher + incremental re-JIT on change. Shares infrastructure with `xiom-codegen` hot-reload system. | 3d | M10.1 |

**Total M10 effort: 18d for complete production implementation.**

## 8. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLVM JIT initialization latency (100-200ms) | Slower than Python for Hello World | Lazy compilation (only compile main, not entire stdlib). Cranelift fallback for O0 tier. |
| inkwell API surface incomplete for ORC JITv2 | Missing features needed for self-host | Verify inkwell 0.5 supports our needs before starting. Fallback: raw llvm-sys bindings. |
| Implicit main wrapping changes semantics | User confusion about scope | Only wrap when NO explicit main exists. `--no-implicit-main` flag to disable. |
| JIT and AOT produce different results | Trust erosion | Differential testing: JIT and AOT must produce identical stdout/stderr/exit code. Part of CI. |
| Self-host compiler too large for JIT | OOM, slow | Incremental JIT: compile only called functions. Lazy compilation built into ORC JITv2. |

## 9. Success Criteria (All Mandatory)

1. `echo 'io.println("hello")' | xiom run -` prints "hello" with <30ms wall time
2. `xiom run selfhost/lexer.xi < test.xi` lexes correctly via JIT, output byte-identical to AOT
3. `xiom run selfhost/parser.xi < test.xi` parses correctly via JIT, AST identical to AOT
4. `xiom run selfhost/checker.xi < test.xi` type-checks correctly via JIT
5. `xiom run selfhost/codegen.xi < test.xi` emits identical LLVM IR to AOT
6. `xiom run selfhost/compiler.xi examples/demo_float.xi` produces binary that runs and returns same exit code as AOT-compiled selfhost
7. `xiom build --standalone myscript.xi -o mytool.exe` produces a binary that runs identically to `xiom run myscript.xi`
8. `xiom repl` session: define variable, use in expression, shadow it, inspect type -- all correct
9. `xiom run --watch script.xi` re-executes on file save within 50ms
10. All existing 934 tests pass with zero regressions
11. New test suite: 50+ JIT-specific tests covering every phase
12. Shebang scripts: `#!/usr/bin/env xiom` executable scripts work on Linux/macOS
13. `xiom run -e` inline mode handles multi-statement input correctly
14. Contract violations in JIT mode trap with the same file/line/message as AOT mode

## 10. Post-M10 Vision

With JIT/Scripting mode operational:
- XIOM becomes a **single-language stack** -- scripts, tools, and core engines all in XIOM
- AI agents get instant feedback via `xiom run -e` in MCP tool calls
- DevOps replaces Bash/Python scripts with verified, static XIOM scripts
- Self-host development cycle: edit -> `xiom run` -> test -> `xiom build --release` -> deploy
- Interactive data science / exploration via REPL (Phase 3)
- Game engine scripting with hot-reload JIT (Phase 3)
