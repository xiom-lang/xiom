# XIOM Compiler Architecture

**Version:** v0.50.0 | **Date:** 2026-07-25
**Tests:** 1041/1041 | **Crates:** 17 | **Stdlib:** 40 modules

---

## 1. Pipeline

```
.xi Source
    │
    ▼
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌───────────┐    ┌──────┐
│  Lexer   │───▶│  Parser  │───▶│  Checker │───▶│Borrow Checker│───▶│  Codegen  │───▶│clang │──▶ binary
│xiom-lexer│    │xiom-parser│   │xiom-check│   │ (check crate)│   │xiom-codegen│   │      │
└──────────┘    └──────────┘    └──────────┘    └──────────────┘    └───────────┘    └──────┘
     │               │               │                                    │
     ▼               ▼               ▼                                    ▼
  TokenStream      Program        Checked              Optional:       LLVM IR (.ll)
  Vec<Token>     (AST root)      Program           ┌──────────┐    "; XIOM v0.50.0 LLVM IR"
                                                    │ xiom-    │    target triple = "..."
                                                    │ verify   │    define i64 @main(...)
                                                    └────┬─────┘
                                                         ▼
                                                    SMT-LIB 2.6
                                                    (Z3 proof)
```

### What each stage does

| Stage | Input | Output | Key logic |
|-------|-------|--------|-----------|
| **Lexer** | `.xi` source text | `Vec<Token>` | Character-by-character scan. Identifies keywords, literals, operators. Shebang (`#!`) line skipped. |
| **Parser** | `Vec<Token>` | `Program` (AST) | LL(1) recursive descent. No backtracking. Builds AST with spans for error reporting. `use` resolution through xiom-graph. |
| **Checker** | `Program` | Checked `Program` | Type inference, name resolution, interface satisfaction, generic monomorphisation. Module catalog built from stdlib + source dirs. |
| **Borrow Checker** | Checked `Program` | Validated program | Lexical-scope ownership. Tracks read/write borrows per variable. Errors on use-after-move, double mutable borrow. |
| **Codegen** | Checked `Program` | LLVM IR text | Walks AST, emits IR. Function definitions, expressions, match, control flow. Contract guards become `@llvm.trap()` calls. |
| **clang** | LLVM IR + C runtime | Native binary | Compiles `.ll` → `.o`, links with `xiom_runtime.c`. Produces `.exe`/`.out`/`.wasm`. |

---

## 2. Lexer (`xiom-lexer`)

**File:** `crates/xiom-lexer/src/lib.rs` (503 lines)

```
Source: "fn add(a: Int, b: Int) -> Int { return a + b; }"
   │
   ▼
Tokens: [Fn, Ident("add"), LParen, Ident("a"), Colon, Ident("Int"),
         Comma, Ident("b"), Colon, Ident("Int"), RParen, Arrow,
         Ident("Int"), LBrace, Return, Ident("a"), Plus,
         Ident("b"), Semicolon, RBrace, EOF]
```

### Key types

```rust
pub struct Lexer { source: Vec<char>, pos: usize, line: usize, col: usize }

pub struct Token { pub kind: TokenKind, pub lexeme: String, pub span: Span }

pub enum TokenKind {
    // Keywords
    Fn, Let, Var, Return, If, Else, Elif, Match, While, For, In,
    Module, Use, Pub, Type, Enum, Interface, Derive, Async, Await,
    Spawn, Unsafe, Extern, Comptime, As, Is, Where, And, Or, Not,
    // Literals
    Ident(String), Int(u64), Float(f64), Str(String), Char(char),
    Bool(bool), Self_, Some, None, Ok_, Err_,
    // Operators & delimiters
    Plus, Minus, Star, Slash, Percent, Eq, Neq, Lt, Gt, Le, Ge,
    AndAnd, OrOr, Bang, Dot, Comma, Colon, Semicolon, Arrow,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    // Special
    Error(String), Eof,
}
```

### Design decisions

- **No regex.** Character-by-character matching. Every operator and keyword is matched explicitly.
- **Shebang support.** If source starts with `#!`, the first line is skipped as a comment.
- **Span tracking.** Every token carries `(line, col)` for accurate error messages.
- **No allocation for keywords.** Keywords are matched from a static list, not interned.

---

## 3. Parser (`xiom-parser`)

**File:** `crates/xiom-parser/src/lib.rs` (1,851 lines)

### Architecture

```
LL(1) recursive descent — one `parse_*` function per grammar production.

parse_program()          → Vec<TopDecl>
  parse_top_decl()       → TopDecl
    parse_fn_decl()      → FnDecl (if peek == Fn)
    parse_type_decl()    → TypeDecl (if peek == Type)
    parse_enum_decl()    → EnumDecl (if peek == Enum)
    parse_interface()    → InterfaceDecl (if peek == Interface)
    parse_module()       → Module (if peek == Module)
    parse_use()          → UseDecl (if peek == Use)
    parse_const()        → ConstDecl (if peek == Const | Pub + Const)
    parse_extern()       → ExternDecl (if peek == Extern)

parse_expr()             → Expr
  parse_assignment()     → Assign | CompoundAssign
  parse_logical_or()     → Or
  parse_logical_and()    → And
  parse_comparison()     → Eq | Neq | Lt | Gt | Le | Ge
  parse_additive()       → Plus | Minus
  parse_multiplicative() → Star | Slash | Percent
  parse_unary()          → Neg | Not | Ref | Deref
  parse_postfix()        → Call | Index | Field | Dot
  parse_atom()           → Ident | Int | Float | Str | Char | Bool | LParen | LBrace | If | Match
```

### Key types

```rust
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    in_loop: bool,        // Tracks if inside a loop (for break/continue validation)
    current_module: Option<String>,
}

pub enum TopDecl {
    Fn(FnDecl), Type(TypeDecl), Enum(EnumDecl), Interface(InterfaceDecl),
    Module(ModuleDecl), Use(UseDecl), Const(ConstDecl), Extern(ExternDecl),
}

pub enum Expr {
    Int(u64, Span), Float(f64, Span), Str(String, Span), Bool(bool, Span),
    Ident(Ident), Binary(Box<Expr>, BinOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span), Call(Box<Expr>, Vec<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span), Field(Box<Expr>, Ident, Span),
    If(Box<Expr>, Block, Vec<(Expr, Block)>, Option<Block>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    Block(Block), Assign(Box<Expr>, Box<Expr>, Span),
    Closure(FnDecl, Span), Array(Vec<Expr>, Span),
    Struct(Ident, Vec<(Ident, Expr)>, Span),
    // ... 20+ variants total
}
```

### Error recovery

- Single-error termination. First parse error aborts with span information.
- Error messages include expected token set: `expected one of: fn, type, enum, ...`

---

## 4. Type Checker (`xiom-check`)

**File:** `crates/xiom-check/src/lib.rs` (4,415 lines — being split in M14)

### Architecture

```
Checker
├── types: TypeContext
│   ├── structs: HashMap<String, Vec<(String, String)>>    // type → fields
│   ├── functions: HashMap<String, FnSig>                   // fn → signature
│   ├── interfaces: HashMap<String, InterfaceDef>           // interface → methods
│   ├── enum_variants: HashMap<String, Vec<Variant>>        // enum → variants
│   └── impls: HashMap<String, HashSet<String>>             // interface → concrete types
├── scopes: Vec<Scope>                                     // lexical scopes
├── current_fn: Option<FnSig>                              // for return type validation
├── errors: Vec<CheckError>
└── source_dirs: Vec<String>                               // for module resolution

check_program(program)
├── register_top_level(program.items)     // Phase 1: collect all declarations
├── check_module_items(program.items)     // Phase 2: type-check bodies
├── resolve_imports()                     // Phase 3: resolve use chains
└── check_interface_satisfaction()        // Phase 4: verify interface impls

check_expr(expr, expected_type)
├── Int/Float/Bool/Str/Char → literal types
├── Ident → lookup in scope → return declared type
├── Binary(lhs, op, rhs) → check lhs, check rhs → op return type
├── Call(func, args) → resolve func → check args against params
├── If/Match → unify branch types
├── Field(obj, name) → resolve obj type → find field
├── Index(obj, idx) → resolve obj → element type
└── Struct(name, fields) → resolve struct → check fields
```

### Type Compatibility

```rust
fn types_compatible(found, expected) -> bool {
    // Exact match
    if found == expected { return true; }
    // impl Trait accepts any type (M9.6)
    if matches!(found/expected, ImplTrait) { return true; }
    // Numeric coercion: Int8/Int16/... ↔ Int64
    if both are numeric { return true; }
    // Struct/Enum name match
    if both are Named { return name == name; }
    // Generic substitution
    if type_map contains generic { substitute and recurse; }
    // Error propagation
    if found == Error { return true; }
    false
}
```

### Module Resolution

```
use xiom.io;        → Check XIOM_STDLIB/xiom/io.xi
use xiom.math;      → Check XIOM_STDLIB/xiom/math.xi
use ./utils;        → Check relative to source file
use mypkg;          → Check ~/.xiom/packages/mypkg/src/

Source dirs (checked in order):
1. Source file's directory
2. Stdlib directory (auto-discovered from binary path or XIOM_STDLIB)
3. Package directories (~/.xiom/packages/*)
4. Project graph roots (package.xi dependencies)
```

### Borrow Checker

The borrow checker is a sub-module of `xiom-check`. It uses lexical-scope ownership:

```rust
pub struct BorrowChecker {
    loans: HashMap<String, LoanState>,     // per-variable borrow tracking
    moved: HashSet<String>,                 // variables consumed by move
}

enum LoanState {
    None,                                   // not borrowed
    Read(Vec<Span>),                        // one or more read borrows
    Write(Span),                            // exactly one write borrow
}
```

**Rules enforced:**
- Use after move → error
- Write borrow while read borrows active → error
- Multiple write borrows → error
- Move on call → old binding invalidated

---

## 5. Code Generation (`xiom-codegen`)

**Files:** 14 source files, ~13,000 lines total

### Architecture (M4.1 — God Object Decomposed)

The IrEmitter struct was decomposed from 86 fields into 5 sub-contexts:

```rust
pub struct IrEmitter {
    // Output
    output: String,             // Accumulated LLVM IR text
    tmp_counter: u32,           // Unique temp name counter
    block_counter: u32,         // Unique block label counter
    str_counter: u32,           // Unique string constant counter

    // Sub-contexts (M4.1)
    config: CodegenConfig,      // Target triple, check_contracts, strict_mode, hot_reload
    types: TypeContext,         // Type registry, struct field layouts, function signatures
    fctx: FunctionContext,      // Current function: locals, params, return type, ensures
    mono: MonoContext,          // Monomorphisation: generic_fn_decls, type_map, emitted_fns
    local: LocalContext,        // Variable classification: bool_locals, ptr_locals, loop_stack
}
```

### IR Emission Flow

```
compile_program(program)
├── compile_module_decls()          // Type/enum/interface declarations
├── compile_derive_impls()          // derive[Eq, Clone, Display, Hash, Ord]
├── compile_fn_decls()              // Forward declarations (for mutual recursion)
├── for each fn:
│   ├── compile_fn_header()         // define i64 @fn_name(params...)
│   ├── compile_fn_body()           // allocas, expressions, control flow
│   │   ├── compile_stmt()          // Let, Var, Assign, Return, If, Match, While, For
│   │   └── compile_expr()          // Int, Float, Binary, Call, Field, Index, Struct
│   └── compile_fn_epilogue()       // Return, close block
├── compile_monomorphised_fns()     // Generic instantiations (two-pass)
└── emit_module_footer()            // Module-level globals, string constants
```

### LLVM Type Mapping

| XIOM Type | LLVM IR | Notes |
|-----------|---------|-------|
| `Int` | `i64` | Default integer |
| `Int8`-`Int64` | `i8`-`i64` | Explicit width |
| `Float32` | `float` | |
| `Float64` | `double` | Default float |
| `Bool` | `i1` | Stored as `i64` in structs |
| `Str` | `i8*` | Pointer to UTF-8 buffer |
| `Char` | `i32` | Unicode code point |
| `*T` / `&T` | `i64` | Pointers lowered to integer |
| `Option[T]` | `{ i64, i64 }` | discriminant + value |
| `Result[T,E]` | `{ i64, i64, i64 }` | discriminant + value + error |
| `Vec[T]` | `{ i8*, i64, i64, i64 }` | data ptr, len, cap, elem_size |
| `struct` | `%struct.Name { ... }` | Named LLVM struct |
| `enum` | `{ i64, i64 }` | discriminant + union payload |
| `[N]T` | `{ i64, [N x type] }` | length + fixed array |

### Contract Codegen

```xiom
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures: result * b == a
```

Emits:

```llvm
define double @divide(double %a, double %b) {
  ; requires guard
  %req_ok = fcmp une double %b, 0.0
  br i1 %req_ok, label %body, label %trap
body:
  %result = fdiv double %a, %b
  ; ensures guard
  %ens_check = fmul double %result, %b
  %ens_ok = fcmp oeq double %ens_check, %a
  br i1 %ens_ok, label %return, label %trap
trap:
  call void @llvm.trap()
  unreachable
return:
  ret double %result
}
```

### Monomorphisation

Two-pass system for generic functions:

```
Pass 1 (Register): Walk program, collect generic_fn_decls
Pass 2 (Specialize): For each concrete instantiation:
  1. Build type_map: { T → Int, U → Str, ... }
  2. Clone AST with substitutions
  3. Emit specialized function: fn_name_Int_Str
  4. Track in emitted_fns to avoid duplicates
```

---

## 6. Compilation Modes

### AOT (Ahead-of-Time)

Standard path. Lex → Parse → Check → Codegen → clang → binary.

```bash
xiom main.xi -o app.exe
xiom main.xi --release
xiom main.xi --target wasm
```

### Scripting (`xiom run`)

Same pipeline with pre-processing:

```
1. Shebang strip:         #!/usr/bin/env xiom → skipped
2. Implicit main wrap:    io.println("hi") → fn main() { io.println("hi") }
3. Auto-import:           no use xiom.io → use xiom.io; added
4. Declarations stay:     type/enum/fn/module/use stay at top level
5. Compile:               Same AOT pipeline
6. Execute:               Run temp binary (--jit: load as DLL, call main() in-process)
7. Cache:                 Content-hash → ~/.xiom/jit/ (100 MB LRU eviction)
```

```bash
xiom run script.xi
xiom run -e "io.println(42)"
xiom run -                                 # stdin
xiom run --watch script.xi                 # auto re-run on change
```

### Standalone (`xiom --standalone`)

Graduate a script to a production binary:

```
Script source → implicit main wrap → compile with --release → standalone binary
xiom --standalone script.xi -o mytool.exe
xiom --standalone --scaffold script.xi     # also create project directory
```

### Check-Only (`xiom --check`)

Type-check without codegen. Used in CI/IDEs:

```
Lex → Parse → Check → Done (no IR, no binary)
xiom --check file.xi
xiom --check --diagnostics-json file.xi
```

### Emit IR (`xiom --emit-ir`)

Print LLVM IR without linking. Used for debugging and playground:

```
Lex → Parse → Check → Codegen → Print IR to stdout
; XIOM v0.50.0 LLVM IR
; Auto-generated by xiom
target triple = "x86_64-pc-windows-msvc"
define i64 @main(...)
```

### WASM (`--target wasm`)

Same pipeline, different clang target triple:

```
Lex → Parse → Check → Codegen → clang --target=wasm32-unknown-unknown → .wasm
```

The playground WASM compiler (`xiom-wasm`) only does Lex→Parse→Check→IR — no clang/linking in browser.

---

## 7. Crate Structure

```
xiom                    CLI entry point + library API
├── xiom-ast            AST node definitions (604 lines)
├── xiom-lexer          Tokenizer (503 lines)
├── xiom-parser         Recursive descent parser (1,851 lines)
├── xiom-check          Type checker + borrow checker (4,415 lines, 7 files)
│   ├── borrow/         Borrow checking (loans, places)
│   └── compat/         Type compatibility/coercion
├── xiom-codegen        LLVM IR generation (13,000 lines, 14 files)
│   ├── context.rs      CodegenConfig, TypeContext, FunctionContext, MonoContext, LocalContext
│   ├── decl.rs         Function/struct/enum declarations
│   ├── stmt.rs         Statement compilation (extracted from expr.rs, M4.2)
│   ├── expr.rs         Expression compilation
│   ├── call.rs         Function/method call compilation (extracted from expr.rs, M4.2)
│   ├── types.rs        Type resolution, LLVM type mapping
│   ├── coerce.rs       Value coercion (inttoptr, bitcast, etc.)
│   ├── contracts.rs    Contract guard emission
│   ├── vec_abi.rs      Vec ABI primitives
│   ├── enum_ctors.rs   Enum constructor generation
│   ├── emitter.rs      IR output helpers
│   ├── sandbox.rs      Safety auditor
│   └── jit.rs          JIT via libloading
├── xiom-graph          Project dependency graph (7 files)
├── xiom-verify         SMT-LIB generation for Z3 (863 lines)
├── xiom-fmt            Source formatter (1,123 lines)
├── xiom-display        Type/fn signature display (121 lines)
├── xiom-doc            Documentation generator
├── xiom-lsp            Language server (10 modules, 2,850 lines)
│   ├── backend         Document storage + diagnostics
│   ├── transport       JSON-RPC stdin/stdout I/O
│   ├── handlers        Method dispatch (hover, completion, definition, etc.)
│   ├── resolver        Type resolution + symbol lookup
│   ├── symbols         Document/workspace symbols
│   └── semantic_tokens, ai, text_edit, uri, diagnostics
├── xiom-mcp            MCP server (Model Context Protocol)
├── xiom-pkg            Package manager (install, search)
├── xiom-dbg            DAP debug server (GDB/MI + CDB backends)
├── xiom-ffigen         FFI bindings generator
└── xiom-wasm           WASM compiler for playground (142 lines)
```

---

## 8. LSP Architecture

```
VS Code / Editor
     │  JSON-RPC (stdin/stdout)
     ▼
┌────────────────────────────────────┐
│            xiom-lsp                 │
│                                     │
│  transport.rs  ←→  stdin/stdout     │  I/O layer
│       │                             │
│       ▼                             │
│  main.rs (dispatch)                 │  Routes method → handler
│       │                             │
│       ├── initialize               │
│       ├── shutdown                 │
│       ├── textDocument/didOpen     │──→ backend.rs (document storage)
│       ├── textDocument/didChange   │
│       ├── textDocument/didClose    │
│       ├── textDocument/hover       │──→ handlers.rs + resolver.rs
│       ├── textDocument/completion  │──→ handlers.rs + resolver.rs
│       ├── textDocument/definition  │──→ handlers.rs + symbols.rs
│       ├── textDocument/signatureHelp│
│       ├── textDocument/documentSymbol│──→ symbols.rs
│       ├── textDocument/references  │
│       ├── textDocument/rename      │
│       ├── textDocument/semanticTokens│──→ semantic_tokens.rs
│       ├── textDocument/codeAction  │──→ handlers.rs
│       └── workspace/symbol        │──→ symbols.rs
│                                     │
│  backend.rs                         │  Document store + publishDiagnostics
│  ai.rs                              │  AI insight integration
│  text_edit.rs                       │  Incremental text sync
└────────────────────────────────────┘
```

### Completion flow

```
User types "io."  →  textDocument/completion
  1. extract_word() → current prefix
  2. Detect dot → obj_name = "io", member_prefix = ""
  3. Keywords + snippets for non-dot completions
  4. Dot completion:
     a. resolve_obj_type_text("io") → module
     b. collect_module_members("io", "") → io functions
     c. Return: println, print, read_line, read_file, ...
  5. Future (M13): pre-built stdlib_completions.json catalog
```

---

## 9. Standard Library

**Location:** `stdlib/xiom/` — 40 modules, ~113 contracts

### Module Organization

| Category | Modules |
|----------|---------|
| **Core Types** | core (Option, Result, Box, interfaces, intrinsics) |
| **Collections** | collections (Vec, Map, Set, Slice, Queue, Stack, BTreeMap) |
| **I/O** | io (console, files, process, paths) |
| **Strings** | string (UTF-8 ops), fmt (formatting) |
| **Math** | math (sqrt, pow, trig), num (traits, checked ops) |
| **Memory** | mem (swap, replace, drop), alloc (Layout, Allocator), ptr (raw pointers) |
| **Concurrency** | sync (Mutex, RwLock, Arc, Atomic, Condvar, Barrier), thread (spawn, JoinHandle), async (Executor, Channel) |
| **Time** | time (Duration, Instant, SystemTime, DateTime) |
| **System** | os (platform, process, signals, pipes, file watching), env (environment, directories) |
| **Network** | net (TCP, UDP, HTTP, DNS, URL) |
| **Data** | iter (Range, Iterator adapters), array ([N]T operations), hash, char, convert, cmp, error |
| **Format** | fmt (Display, Formatter), serialize (JSON), encoding (base64, hex) |
| **Crypto** | crypto (AES, SHA, Ed25519, PBKDF) |
| **Testing** | test (assert), bench, runner, stats |
| **Paths** | path (Path, PathBuf) |
| **Random** | rand (RNG, distributions) |
| **Logging** | log (levels, formatting) |
| **Compression** | compress (gzip, deflate) |

### Contract Coverage

| Module | Contract count | Key contracts |
|--------|---------------|---------------|
| stats | 13 | Division-by-zero, sqrt domain, array bounds |
| array | 26 | Zero-size slice safety, rotation bounds, sort post-conditions |
| mem | 14 | Uninitialized memory safety, ManuallyDrop invariants |
| fmt | 10 | Formatter write post-conditions |
| runner | 9 | Integer overflow, division by zero |
| io | 8 | File path existence, write success |
| thread | 6 | Non-null thread handles, join invariants |
| time | 4 | Nanosecond normalization, div-by-zero |
| path | 4 | Non-empty paths, parent existence |
| iter | ~8 | nth bounds, count non-negative, sortedness claims |
