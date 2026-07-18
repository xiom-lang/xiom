// XIOM MCP — Language & Workflow Guides
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
        "package" => W_PACKAGE.into(),
        "sandbox" => W_SANDBOX.into(),
        _ => W_OVERVIEW.into(),
    }
}

const OVERVIEW: &str = r#"# XIOM Language Guide — Topics

Call `xiom_language_guide {topic}` with one of:
- `types` — primitives, structs, enums, derive, generics
- `ownership` — move semantics, borrows, clone, common E001 fixes
- `contracts` — requires/ensures/invariant, result, @pre, implication
- `modules` — module decl, use imports, visibility, multi-file builds
- `error-handling` — Option/Result, match, ? operator, traps
- `unsafe-ffi` — extern "C", unsafe blocks, pointer rules, symbol shadowing
- `debugging` — reading diagnostics, --explain codes, common error fixes

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
pub type Shape = enum { Circle(Float64); Rect(Float64, Float64); Empty; }
let s = Shape::Circle(2.0);
match s {
  Shape::Circle(r) => { ... }
  Shape::Rect(w, h) => { ... }
  Shape::Empty => { ... }
}
```

## Generics — square brackets
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

## Common errors → fixes
- E001 "use after move": pass `&value` instead, or `.clone()` first.
- "cannot borrow as mutable while borrowed": narrow borrow scopes.
- Returning `&local`: rejected — return by value instead.
- Struct fields cannot store borrows — structs own their data.

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
- `result` — return value (ensures only)
- `x@pre` — entry value of x (ensures only)
- `a => b` — implication
- `result is Ok => result != null` — pattern implication

## Behavior
- Violation → trap with contract name.
- `xiomc --no-contracts` disables runtime checks.
- `xiomc --dump-contracts` exports JSON (also: get_contract_signature MCP tool).

## Best practices
1. Null/range checks belong in requires — callers see them in the signature.
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
File's own dir → ./stdlib → XIOM_STDLIB env → exe-adjacent stdlib/ (release).
A catalog index gives O(1) module lookup.

## Visibility
`pub` = exported. Private items are module-internal.

## Multi-file builds
`xiomc -o app.exe main.xi lib.xi extra.xi` — files merge into one program;
cross-file types resolve automatically.

## Errors
- "unknown module" → file not in a scanned dir; pass on CLI or set XIOM_STDLIB.
- private access → add `pub` at the declaration."#;

const ERROR_HANDLING: &str = r#"# XIOM Error Handling

## Option[T]
```xiom
fn find(v: &Vec[Int], x: Int) -> Option[Int] {
  var i = 0;
  while i < v.len() {
    if v[i] == x { return Option::Some(i); }
    i = i + 1;
  }
  return Option::None;
}
let idx = find(&v, 42).unwrap_or(0);
```

## Result[T, E]
```xiom
fn parse(s: Str) -> Result[Int, Str] {
  if s.len() == 0 { return Result::Err("empty"); }
  return Result::Ok(0);
}
let n = parse(input)?;   // ? unwraps Ok or early-returns Err
```

## Guidance
- Recoverable failures → Result.
- Absence → Option.
- Programmer errors / invariant violations → contracts (trap on violation)."#;

const UNSAFE_FFI: &str = r#"# XIOM Unsafe & C FFI

## Declare foreign functions
```xiom
extern "C" {
  fn malloc(size: UInt) -> *mut UInt8;
  fn free(ptr: *mut UInt8);
}
```

## Call inside unsafe
```xiom
pub fn alloc(size: Int) -> *mut UInt8
  requires: size > 0;
  ensures:  result != null;
{
  unsafe { return malloc(size as UInt); }
}
```

## Rules
1. Every extern call requires `unsafe { }`.
2. Raw deref `*ptr` requires unsafe.
3. `unsafe { ...; return x; }` as the whole body is fine — divergence analysis
   accepts blocks where every path returns.
4. NEVER name a wrapper after a C symbol (e.g. `pub fn realloc`) — it collides
   with the extern declaration and is silently dropped. Use `realloc_sized`.

## Safety audit
`xiomc --sandbox file.xi` scores unsafe blocks:
- extern call without contract → HIGH
- pointer arithmetic without bounds → HIGH
- unsafe in pub fn → MEDIUM
CI gate: `xiomc --sandbox=strict` (exit 3 on HIGH). JSON: `--sandbox-report=json`."#;

const DEBUGGING: &str = r#"# XIOM Debugging Guide

## Reading compiler errors
Format: `error[CODE]: line:col: message` + note/help lines.
Codes: L001 lexer, P001 parser, T001 types, E001 borrow/move, C001 codegen.
`xiomc --explain T001` prints the full reference for a code.
MCP: explain_error_code {code}.

## Frequent errors → fixes
- T001 "return type mismatch: expected X, found ()" — a code path falls off
  the end. Ensure every path returns (if/else both branches).
- E001 "use after move" — borrow (&x) or clone before the move.
- P001 "expected one of ..." — check semicolons and brackets; generics use [ ].
- "unknown module" — add the file to the CLI invocation or fix `use` path.

## Runtime debugging
1. Compile with symbols: `xiomc -g -o app.exe main.xi`
2. VS Code: install the XIOM extension, F5 with type "xiom" (uses xiom-dbg + GDB).
3. Contract violations trap — the debugger's exception filter
   "Contract Violations" breaks at the violating check.

## Structured output for tools
`xiomc --diagnostics=json file.xi` — machine-readable diagnostics.
MCP compile_and_analyze returns the same structure."#;

const W_OVERVIEW: &str = r#"# XIOM Toolchain Workflows

Call `xiom_workflow_guide {topic}` with one of:
- `compile` — build binaries, IR, WASM; flags reference
- `test` — write and run XIOM tests
- `debug` — symbols, debugger, contract traps
- `package` — package.xi manifest, lockfile, publish to registry
- `sandbox` — safety audit + CI/CD gating

## Toolchain binaries
| Tool | Purpose |
|------|---------|
| xiomc | Compiler (native, WASM, IR) |
| xiom-fmt | Canonical formatter (--check, --in-place) |
| xiom-lsp | Language server (editors) |
| xiom-dbg | DAP debug adapter (VS Code/JetBrains) |
| xiom-pkg | Package manager (install/publish/lock) |
| xiom-doc | Markdown doc generator |
| xiom-ffigen | C header → XIOM bindings |
| xiom-verify | SMT-LIB contract export (Z3) |
| xiom-mcp | This MCP server |"#;

const W_COMPILE: &str = r#"# Compile Workflows

## Basic
xiomc file.xi                    # print LLVM IR to stdout
xiomc -o app.exe main.xi         # native binary
xiomc --run main.xi              # compile + run, prints exit code
xiomc -o app.exe a.xi b.xi c.xi  # multi-file

## Targets
xiomc --target wasm -o app.wasm main.xi
xiomc --target arm / riscv       # cross-compile triples

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
xiomc --run tests.xi
echo $LASTEXITCODE   # 0 = green

## Compiler's own suite (contributors)
./test_summary.ps1   (Windows)  |  ./test_summary.sh  (Unix)
cargo test --all     # 716 tests: compiler 495 + tooling 221"#;

const W_DEBUG: &str = r#"# Debug Workflows

## 1. Compile with symbols
xiomc -g -o app.exe main.xi

## 2. VS Code (XIOM extension)
launch.json:
{
  "type": "xiom", "request": "launch",
  "name": "Debug XIOM",
  "program": "${workspaceFolder}/app.exe",
  "stopOnEntry": true, "contractTraps": true
}
Uses xiom-dbg (DAP) over GDB/MI: breakpoints, step, locals, stack.

## 3. Contract traps
A violated requires/ensures traps. With contractTraps: true the debugger
breaks there; the IR carries `; contract:` comments mapping trap → clause.

## 4. CLI fallback
gdb ./app.exe  (DWARF symbols work in any GDB-compatible debugger)"#;

const W_PACKAGE: &str = r#"# Package & Publish Workflows

## Manifest: package.xi at project root
package {
  name: "mylib";
  version: "0.1.0";
  description: "What it does";
  authors: ["you"];
  modules: ["src/lib.xi"];
  deps: { xiom-std: "0.47.0"; }
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
Lockfile (xiom.lock) pins dep versions — commit it."#;

const W_SANDBOX: &str = r#"# Safety Audit (Sandbox) Workflows

## Run
xiomc --sandbox file.xi                  # human-readable report
xiomc --sandbox-report=json file.xi      # JSON (CI parsing)
xiomc --sandbox-report=out.json file.xi  # write to file
xiomc --sandbox=strict file.xi           # exit 3 if HIGH findings

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
- run: xiomc --sandbox=strict src/main.xi"#;
