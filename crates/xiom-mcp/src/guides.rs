// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM MCP -- Language & Workflow Guides
// Deep curated reference for agents with no XIOM training data.

/// Deep language semantics by topic.
pub fn language_guide(topic: &str) -> String {
    match topic {
        "ownership" => OWNERSHIP.into(),
        "contracts" => CONTRACTS.into(),
        "unsafe-ffi" => UNSAFE_FFI.into(),
        "modules" => MODULES.into(),
        "error-handling" => ERROR_HANDLING.into(),
        "types" => TYPES.into(),
        "debugging" => DEBUGGING.into(),
        _ => OVERVIEW.into(),
    }
}

/// Toolchain workflow reference.
pub fn workflow_guide(topic: &str) -> String {
    match topic {
        "compile" => W_COMPILE.into(),
        "test" => W_TEST.into(),
        "debug" => W_DEBUG.into(),
        "script" => W_SCRIPT.into(),
        "package" => W_PACKAGE.into(),
        "sandbox" => W_SANDBOX.into(),
        _ => W_OVERVIEW.into(),
    }
}

const OVERVIEW: &str = r#"# XIOM Language Guide -- Topics

Call `xiom_language_guide {topic}` with one of:
- `types` -- primitives, structs, enums, derive, generics
- `ownership` -- move semantics, borrows, clone, common E001 fixes
- `contracts` -- requires/ensures/invariant, result, @pre, implication
- `modules` -- module decl, use imports, visibility, multi-file builds
- `error-handling` -- Option/Result, match, ? operator, traps
- `unsafe-ffi` -- extern "C", unsafe blocks, pointer rules, symbol shadowing
- `debugging` -- reading diagnostics, --explain codes, common error fixes

Quick facts:
- Files end in `.xi`. Entry point: `fn main() -> Int`.
- Statements end with `;`. Blocks are `{ }`. Types after names: `x: Int`.
- Generics use SQUARE brackets: `Vec[Int]`, `fn first[T](...)`.
- No garbage collector: ownership + ARC where needed.
- Contracts are built into the language, not comments."#;

const TYPES: &str = r#"# XIOM Type System

## Primitives
Int (i64), Int32, UInt (u64), UInt8/16/32/64, Float32, Float64, Bool, Char, Str, Unit

## Structs
```xiom
pub type Point = { x: Float64; y: Float64; } derive[Eq, Clone]
let p = Point{ x: 1.0, y: 2.0 };
let d = p.x;
```
derive options: Eq, Clone, Display, Hash, Ord

## Enums (sum types)
```xiom
pub type Shape = enum { Circle(r: Float64), Rect(w: Float64, h: Float64), Empty, }
let s = Shape.Circle(2.0);        // construct with TypeName.Variant(...)
match s {
  Circle(r) => { ... }            // match with BARE variant patterns
  Rect(w, h) => { ... }
  Empty => { ... }
}
```
Note: variants separated by COMMAS (trailing comma ok); payload fields are
NAMED (`Circle(r: Float64)`). Construction uses DOT syntax (`Shape.Circle(2.0)`),
never `Shape::Circle`. Match patterns use bare variant names.
`derive[Clone, Eq]` works on enums including heap payloads (Str/Vec).

## Generics -- square brackets
```xiom
fn first[T](items: &Slice[T]) -> T { return items[0]; }
fn max[T: Ord](a: T, b: T) -> T { if a > b { return a; } return b; }
```

## Casts
`x as Int32`, `f as Int` (truncates), `ptr as *Float32`, `(if c {a} else {b}) as Int32`

## Type aliases + invariants
```xiom
pub type Meters = Float64
pub type NonEmpty = Str invariant: this.len() > 0;
```"#;

const OWNERSHIP: &str = r#"# XIOM Ownership & Borrowing

XIOM uses move semantics with borrow checking.

## Rules
1. Every value has exactly ONE owner.
2. Assignment and by-value calls MOVE ownership; the source becomes unusable.
3. `&x` = shared read borrow (many allowed).
4. `&mut x` = exclusive write borrow (only one; no reads while active).
5. Borrows end at scope end.
6. `.clone()` deep-copies, keeping the original usable.

## Common errors -> fixes
- E001 "use after move": pass `&value` instead, or `.clone()` first.
- "cannot borrow as mutable while borrowed": narrow borrow scopes.
- Returning `&local`: rejected -- return by value instead.
- Struct fields cannot store borrows -- structs own their data.

## Patterns
```xiom
fn read_only(data: &Vec[Int]) -> Int { return data.len(); }
fn consume(data: Vec[Int]) { ... }
let copy = original.clone();
consume(copy);          // move the clone
read_only(&original);   // original still usable
```"#;

const CONTRACTS: &str = r#"# XIOM Contracts (Design by Contract)

First-class: runtime-checked (debug), statically verifiable (Z3, 5f), tool-queryable.

## Syntax
```xiom
fn divide(a: Int, b: Int) -> Int
  requires: b != 0;
  ensures:  result * b == a;
{
  return a / b;
}

pub type PositiveInt = Int invariant: this > 0;
```

## Special forms
- `result` -- return value (ensures only)
- `x@pre` -- entry value of x (ensures only)
- `a => b` -- implication
- `result is Ok => result != null` -- pattern implication

## Behavior
- Violation -> trap with contract name.
- `xiom --no-contracts` disables runtime checks.
- `xiom --dump-contracts` exports JSON (also: get_contract_signature MCP tool).

## Best practices
1. Null/range checks belong in requires -- callers see them in the signature.
2. FFI wrappers MUST declare `ensures: result != null` or handle failure.
3. Contract expressions must be pure (no side effects)."#;

const MODULES: &str = r#"# XIOM Modules & Imports

## Declare (first line of file)
```xiom
module xiom.mymod
```

## Import
```xiom
use xiom.core;      // then: core.print("hi");
use xiom.string;
```

## Resolution order
File's own dir -> ./stdlib -> XIOM_STDLIB env -> exe-adjacent stdlib/ (release).
A catalog index gives O(1) module lookup.

## Visibility
`pub` = exported. Private items are module-internal.

## Multi-file builds
`xiom -o app.exe main.xi lib.xi extra.xi` -- files merge into one program;
cross-file types resolve automatically.

## Errors
- "unknown module" -> file not in a scanned dir; pass on CLI or set XIOM_STDLIB.
- private access -> add `pub` at the declaration."#;

const ERROR_HANDLING: &str = r#"# XIOM Error Handling

## Option[T] -- bare Some/None constructors
```xiom
fn find(v: &Vec[Int], x: Int) -> Option[Int] {
  var i = 0;
  while i < v.len() {
    if v[i] == x { return Some(i); }
    i = i + 1;
  }
  return None;
}
match find(&v, 42) {
  Some(idx) => { core.print("found"); }
  None => { core.print("missing"); }
}
let idx = find(&v, 42).unwrap_or(0);
```

## Result[T, E] -- bare Ok/Err constructors
```xiom
fn parse(s: Str) -> Result[Int, Str] {
  if s.len() == 0 { return Err("empty"); }
  return Ok(0);
}
let r = parse(input);
match r {
  Ok(n) => { use_value(n); }
  Err(msg) => { core.print(msg); }
}
// Accessors:
if r.is_ok() { let n = r.unwrap(); }
if r.is_err() { let msg = r.unwrap_err(); }
let n = parse(input)?;   // ? unwraps Ok or early-returns Err
```

IMPORTANT: constructors are BARE -- `Ok(x)`, `Err(e)`, `Some(x)`, `None`.
There is NO `Result::Ok` / `Option::Some` path syntax in XIOM.

## Guidance
- Recoverable failures -> Result.
- Absence -> Option.
- Programmer errors / invariant violations -> contracts (trap on violation)."#;

const UNSAFE_FFI: &str = r#"# XIOM Unsafe & C FFI (v0.57 Unsafe Confinement)

## Declare foreign functions
```xiom
extern "C" {
  fn malloc(size: UInt) -> *mut UInt8;
  fn free(ptr: *mut UInt8);
}
```

## Call inside unsafe (confined block)
```xiom
pub fn alloc(size: Int) -> *mut UInt8
  requires: size > 0;
  ensures:  result != null;
{
  unsafe { return malloc(size as UInt); }
}
```

## Rules (v0.57 -- all enforced by the compiler)
1. `unsafe` applies STRICTLY to the block `{ }` -- `unsafe fn/module/struct` is a
   hard error.
2. Every extern call requires `unsafe { }` (T002). Exempt: fns declaring
   `requires`/`ensures` contracts (safe-wrapper pattern).
3. Raw deref `*ptr` and Int<->Ptr casts require unsafe.
4. A safe fn cannot return a raw pointer (T003) unless its body contains unsafe
   (unsafe-internal helper exemption).
5. A fn whose ENTIRE body is one unsafe block must declare `requires` (T007).
6. Confined blocks are TRAPPED: a hardware fault is caught by the SEH/sigsetjmp
   trampoline, retried once on a fresh arena slot, then the block yields a
   recoverable zero (HardwareFault) -- the process never crashes.
7. `#[unsafe_no_retry]` disables the once-only transient retry; `#[unsafe_direct]`
   is the trusted escape hatch (stdlib/selfhost; `--enable-unsafe-direct` for
   user code).
8. Zero-escape (T005): raw ptrs / refs / fn types cannot be an unsafe block's
   tail; Str tails are Copy-Out'd to the main heap before the arena resets.
9. FFI ownership (T006): an extern-returned `*T` in a confined block must be
   converted to an owned XIOM type before the tail (ffi.safe_ptr_from_raw /
   box_from_ptr / vec_from_ptr_with_free / str_from_ptr_owned) -- UNLESS the fn
   itself returns the raw pointer (allocator pattern: caller owns it).
10. NEVER name a wrapper after a C symbol (e.g. `pub fn realloc`) -- it collides
    with the extern declaration and is silently dropped. Use `realloc_sized`.
11. `unsafe { ...; return x; }` as the whole body is fine -- divergence analysis
    accepts blocks where every path returns.

## Safety audit
`xiom --sandbox file.xi` scores unsafe blocks (legacy audit -- confinement gates
are the primary layer):
- extern call without contract -> HIGH
- pointer arithmetic without bounds -> HIGH
- unsafe in pub fn -> MEDIUM
CI gate: `xiom --sandbox=strict` (exit 3 on HIGH). JSON: `--sandbox-report=json`
(MCP audit_safety_sandbox passes `--sandbox` first)."#;

const DEBUGGING: &str = r#"# XIOM Debugging Guide

## Reading compiler errors
Format: `error[CODE]: line:col: message` + note/help lines.
Codes: L001 lexer, P001 parser, T001 types, E001 borrow/move, C001 codegen.
`xiom --explain T001` prints the full reference for a code.
MCP: explain_error_code {code}.

## Frequent errors -> fixes
- T001 "return type mismatch: expected X, found ()" -- a code path falls off
  the end. Ensure every path returns (if/else both branches).
- E001 "use after move" -- borrow (&x) or clone before the move.
- P001 "expected one of ..." -- check semicolons and brackets; generics use [ ].
- "unknown module" -- add the file to the CLI invocation or fix `use` path.

## Runtime debugging
1. Compile with symbols: `xiom -g -o app.exe main.xi`
2. VS Code: install the XIOM extension, F5 with type "xiom" (uses xiom-dbg + GDB).
3. Contract violations trap -- the debugger's exception filter
   "Contract Violations" breaks at the violating check.

## Breakpoints
Source-level breakpoints are set by file and line number:
- VS Code: click the gutter next to the line number
- JSON API: `xiom-dbg --json` -> `set-breakpoint main.xi 43`
- GDB directly: `break main.xi:43`
- DAP: `setBreakpoints` request with `source.path` and `breakpoints[].line`

Breakpoint features:
- Source-level: Set at any executable line in a .xi file
- Function entry: break at function prologue
- Contract violation: `contractTraps: true` catches requires/ensures/invariant violations
- Conditional: NOT YET supported (planned Phase 4)
- Hit-count: NOT YET supported (planned Phase 4)

## Stepping
| Command | DAP | JSON API | GDB |
|---------|-----|----------|-----|
| Continue | continue | continue | c |
| Step over | next | step | n |
| Step into | stepIn | step-in | s |
| Pause | pause | -- | Ctrl+C |

## Variable inspection
- VS Code: hover over variable or use Variables panel
- JSON API: `variables` (locals), `evaluate "expr"` (any expression)
- GDB: `info locals`, `print expr`
- Memory: `xiom-dbg --json` -> `memory <addr> <size>` (hex dump)

## Structured output for tools
`xiom --diagnostics=json file.xi` -- machine-readable diagnostics.
MCP compile_and_analyze returns the same structure."#;

const W_OVERVIEW: &str = r#"# XIOM Toolchain Workflows

Call `xiom_workflow_guide {topic}` with one of:
- `compile` -- build binaries, IR, WASM; flags reference
- `test` -- write and run XIOM tests
- `debug` -- symbols, debugger, contract traps
- `package` -- package.xi manifest, lockfile, publish to registry
- `sandbox` -- safety audit + CI/CD gating

## Toolchain binaries
| Tool | Purpose |
|------|---------|
| xiom | Compiler (native, WASM, IR) |
| xiom-fmt | Canonical formatter (--check, --in-place) |
| xiom-lsp | Language server (editors) |
| xiom-dbg | DAP debug adapter (VS Code/JetBrains) |
| xiom-pkg | Package manager (install/publish/lock) |
| xiom-doc | Markdown doc generator |
| xiom-ffigen | C header -> XIOM bindings |
| xiom-verify | SMT-LIB contract export (Z3) |
| xiom-mcp | This MCP server |"#;

const W_COMPILE: &str = r#"# Compile Workflows

## Basic
xiom file.xi                    # print LLVM IR to stdout
xiom -o app.exe main.xi         # native binary
xiom --run main.xi              # compile + run, prints exit code
xiom -o app.exe a.xi b.xi c.xi  # multi-file

## Targets
xiom --target wasm -o app.wasm main.xi
xiom --target arm / riscv       # cross-compile triples

## Modes & flags
--check              type-check only (no codegen)
--emit-ir            print LLVM IR
-g                   debug symbols (DWARF via clang)
--release            optimized build
--no-contracts       strip runtime contract checks
--diagnostics=json   machine-readable errors
--dump-contracts     contract index as JSON
--explain CODE       error-code reference
-l LIB -L PATH       link native libs
--c-source FILE.c    compile+link a C bridge file

## Exit codes
0 success; nonzero = compile/runtime failure (with --run, the program's code)."#;

const W_TEST: &str = r#"# Test Workflows

## Writing tests (xiom.test module)
```xiom
use xiom.test;

fn test_math() -> Int {
  if 2 + 2 != 4 { return 1; }
  return 0;
}

fn main() -> Int {
  var failed = 0;
  failed = failed + test_math();
  return failed;   // 0 = all pass
}
```
Convention: return 0 on success; the process exit code IS the test result.

## Running
xiom --run tests.xi
echo $LASTEXITCODE   # 0 = green

## Compiler's own suite (contributors)
./test_summary.ps1   (Windows)  |  ./test_summary.sh  (Unix)
cargo test --all     # 716 tests: compiler 495 + tooling 221"#;

const W_DEBUG: &str = r#"# Debug Workflows

## 1. Compile with symbols
xiom -g -o app.exe main.xi

## 2. VS Code (XIOM extension)
launch.json:
{
  "type": "xiom", "request": "launch",
  "name": "Debug XIOM",
  "program": "${workspaceFolder}/app.exe",
  "stopOnEntry": true, "contractTraps": true
}
Uses xiom-dbg (DAP) over GDB/MI: breakpoints, step, locals, stack.

## 3. Breakpoints
- VS Code: click gutter next to line number
- JSON API: `xiom-dbg --json` -> `set-breakpoint main.xi 43`
- GDB: `break main.xi:43`
- List: `xiom-dbg --json` -> `breakpoints`
- Delete: `xiom-dbg --json` -> `delete-breakpoint <id>`
Set at any executable .xi line. Function-name breakpoints also work.

## 4. Stepping
- Continue (F5): resume until next breakpoint
- Step over (F10): execute current line, next line
- Step into (F11): enter function call
- Pause: interrupt running program (GDB: Ctrl+C, WinDbg: .break)

## 5. Contract traps
A violated requires/ensures traps. With contractTraps: true the debugger
breaks there; the IR carries `; contract:` comments mapping trap -> clause.

## 6. Variable inspection
- VS Code: Variables panel, hover evaluation
- JSON API: `variables` (locals), `evaluate "expr"` (any expression)
- GDB: `info locals`, `print expr`
- Memory: `memory <addr> <size>` -- hex dump

## 7. Expression evaluation
Supported in all modes: DAP hover, json evaluate, GDB print.
Evaluates arbitrary XIOM expressions in the current stack frame.

## 8. CLI fallback
gdb ./app.exe  (DWARF symbols work in any GDB-compatible debugger)"#;

const W_SCRIPT: &str = r#"# Scripting & JIT Workflows

## xiom run -- execute scripts immediately
xiom run script.xi              # Execute a .xi script (auto-wraps in fn main())
xiom run -e "print(42)"         # Execute inline expression
echo "print(1+1)" | xiom run -  # Execute from stdin
xiom run --watch script.xi      # Watch file, re-run on changes

## Implicit main -- no boilerplate needed
Scripts can write statements directly at the top level. The compiler
automatically wraps them in `fn main()` and adds `use xiom.io;`.

**IMPORTANT: xiom run does NOT relax type checking.** io.println() still
requires Str. Use .to_str() to convert integers: io.println((5+3).to_str()).

## Shebang support
xiom scripts can use #!/usr/bin/env xiom as the first line:
#!/usr/bin/env xiom
io.println("hello from executable script");
chmod +x script.xi && ./script.xi

## xiom --standalone -- script-to-binary
Converts a script into a production binary with --release optimizations:
xiom --standalone script.xi -o mytool
xiom --standalone --scaffold script.xi  # Also create project structure

## Script cache
Repeated runs of the same script are instant -- binaries are content-hash
cached in ~/.xiom/jit/. No recompilation needed.

## AI agent usage (MCP)
When generating XIOM code via MCP, use `xiom run -e "code"` for rapid
testing. The MCP `compile_and_diagnose` tool also accepts scripting-mode
source (it auto-wraps implicit main)."#;

const W_PACKAGE: &str = r#"# Package & Publish Workflows

## Manifest: package.xi at project root
package {
  name: "mylib";
  version: "0.1.0";
  description: "What it does";
  authors: ["you"];
  modules: ["src/lib.xi"];
  deps: { xiom.std: "0.47.0"; }
}

## Commands
xiom-pkg --list --root .        # list modules
xiom-pkg --resolve --root .     # dependency tree
xiom-pkg lock                   # write xiom.lock (reproducible builds)
xiom-pkg install <name>         # fetch from registry
xiom-pkg publish                # publish current package

## Registry
Default: https://registry.xiom-lang.com
Override: XIOM_REGISTRY=https://my-registry.example.com
Lockfile (xiom.lock) pins dep versions -- commit it."#;

const W_SANDBOX: &str = r#"# Safety Audit (Sandbox) Workflows

> v0.57 note: Unsafe Confinement gates (T002/T003/T005/T006/T007) are the PRIMARY
> safety layer -- they make violations COMPILE ERRORS. The sandbox audit below is
> the legacy scoring layer (report + CI gating). Both remain active.

## Run
xiom --sandbox file.xi                  # human-readable report
xiom --sandbox --sandbox-report=json file.xi   # JSON (CI parsing) -- MUST include --sandbox first
xiom --sandbox --sandbox-report=out.json file.xi # write to file
xiom --sandbox=strict file.xi           # exit 3 if HIGH findings

## Exit codes (CI gating)
0 = SAFE/LOW only | 1 = MEDIUM present | 2 = HIGH present | 3 = strict-blocked

## Finding categories
HIGH: extern_c_call_without_contract, raw_pointer_deref,
      raw_pointer_arithmetic, type_punning
MEDIUM: extern_c_call_unchecked_return, large_unsafe_block (>10 stmts),
        unsafe_in_public_api
LOW: unsafe_block_without_comment

## Fixing findings
- Add requires/ensures to the containing fn (removes _without_contract).
- Guard pointer arithmetic with bounds checks.
- Split big unsafe blocks; add `// SAFETY:` comments.

## GitHub Actions example
- run: xiom --sandbox=strict src/main.xi"#;
