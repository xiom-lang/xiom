# XIOM Compiler Architecture

How the XIOM compiler works — pipeline, stages, data structures, and execution modes.

---

## 1. Pipeline

```
                         .xi Source
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
         Script Mode    Check Mode     AOT Mode
        (xiom run)    (xiom --check)  (xiom build)
              │              │              │
              └──────────────┼──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │         LEXER               │
              │   char → token stream       │
              │   strips shebang (#!)       │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │         PARSER              │
              │   token → AST (Program)     │
              │   LL(1) recursive descent   │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │         CHECKER             │
              │   type inference            │
              │   name resolution           │
              │   interface satisfaction    │
              │   borrow checking           │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │         CODEGEN             │
              │   AST → LLVM IR text        │
              │   contract guards emitted   │
              │   monomorphisation          │
              └──────────────┬──────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
           Native         WASM         JIT (DLL)
          clang →      clang →       clang -shared →
         .exe/.out     .wasm      libloading call
```

---

## 2. Lexer

```
Input:  "fn add(a: Int) -> Int { return a + b; }"

Output: [Fn, Ident("add"), LParen, Ident("a"), Colon, Ident("Int"),
         RParen, Arrow, Ident("Int"), LBrace, Return, Ident("a"),
         Plus, Ident("b"), Semicolon, RBrace, EOF]
```

```
Lexer State Machine:

    ┌─────────┐     letter      ┌──────────┐
    │  Start   │───────────────▶│  Ident    │──▶ keyword check
    └─────────┘                 └──────────┘
         │                      ┌──────────┐
         │       digit          │  Number   │──▶ int or float
         ├─────────────────────▶└──────────┘
         │                      ┌──────────┐
         │       "              │  String   │──▶ escape handling
         ├─────────────────────▶└──────────┘
         │                      ┌──────────┐
         │       operator       │ Operator  │──▶ 1-char or 2-char
         ├─────────────────────▶└──────────┘
         │
         └── #! on line 1 → skip to newline (shebang)
```

Key design: no regex, no lookahead. Each character advances the state machine deterministically.

---

## 3. Parser

```
LL(1) Recursive Descent — no backtracking

parse_program()
├── parse_top_decl()          peek → dispatch
│   ├── Fn       → parse_fn_decl()
│   ├── Type     → parse_type_decl()
│   ├── Enum     → parse_enum_decl()
│   ├── Interface→ parse_interface()
│   ├── Module   → parse_module()       (recursive — modules can nest)
│   ├── Use      → parse_use()
│   ├── Const    → parse_const()
│   └── Extern   → parse_extern()
│
└── repeat until EOF

parse_expr() — precedence climbing
│
├── Level 1: Assignment        (=, +=, -=, *=, /=, %=)
├── Level 2: Logical OR        (||)
├── Level 3: Logical AND       (&&)
├── Level 4: Comparison        (==, !=, <, >, <=, >=)
├── Level 5: Additive          (+, -)
├── Level 6: Multiplicative    (*, /, %)
├── Level 7: Unary             (-, !, &, &mut, *)
├── Level 8: Postfix           (call, index, field, dot)
└── Level 9: Atom              (literal, ident, block, if, match)

parse_stmt()
├── Let / Var → parse_let()
├── Return    → parse_return()
├── If/Elif   → parse_if()
├── Match     → parse_match()
├── While     → parse_while()
├── For       → parse_for()
├── Break/Continue
└── Expression → parse_expr() as statement
```

### AST Structure

```
Program
├── TopDecl
│   ├── Fn(FnDecl)           name, params, return_type, body, generics, contracts
│   ├── Type(TypeDecl)       name, fields, derives, invariants
│   ├── Enum(EnumDecl)       name, variants (with optional fields), derives
│   ├── Interface(IfDecl)    name, members (fn signatures, fields)
│   ├── Module(ModDecl)      name, items (recursive Vec<TopDecl>)
│   ├── Use(UseDecl)         path, alias
│   ├── Const(ConstDecl)     name, type, value
│   └── Extern(ExternDecl)   abi (C), fn declarations
│
├── Expr (30+ variants)
│   ├── Literals:      Int, Float, Str, Char, Bool
│   ├── Operators:     Binary(lhs, op, rhs), Unary(op, expr)
│   ├── Control:       If(cond, then, elifs, else), Match(expr, arms)
│   ├── Calls:         Call(func, args), MethodCall(obj, name, args)
│   ├── Access:        Field(obj, name), Index(obj, idx)
│   ├── Construction:  Struct(name, fields), Array(elems)
│   ├── Patterns:      Some(expr), None, Ok(expr), Err(expr)
│   ├── Memory:        Ref(expr), Deref(expr), Unsafe(block)
│   └── Other:         Closure, Block, Assign, CompoundAssign, Range
│
├── Stmt (20+ variants)
│   ├── Let(ident, type, value), Var(ident, type, value)
│   ├── Return(expr), Break, Continue
│   ├── If, Match, While, For
│   └── Expression(expr), Spawn(block)
│
└── Type
    ├── Named(ident, generics)    Int, Vec[Int], Map[Str, Bool]
    ├── Ref(Type), MutRef(Type)
    ├── Option(Type), Result(Type, Type), Vec(Type)
    ├── Ptr(Type), Slice(Type), Map(Type, Type), Set(Type)
    ├── Fn(params, ret)           function pointer
    └── ImplTrait(idents)         impl Display (opaque return, M9.6)
```

---

## 4. Checker

```
Checker
├── type registry
│   ├── structs: name → { fields: (name, type), derives, invariants }
│   ├── functions: name → (param_types, return_type)
│   ├── interfaces: name → { methods, required types }
│   ├── impls: interface → set of concrete type names
│   └── enums: name → [(variant, field_types)]
│
├── scope stack
│   ├── global scope: top-level declarations
│   ├── module scope: module-qualified names
│   └── local scope: function params, let/var bindings
│
├── type checking flow
│   ├── Phase 1: register all declarations (types, functions, interfaces)
│   ├── Phase 2: check function bodies (expressions, statements)
│   ├── Phase 3: resolve use imports (follow module chains)
│   └── Phase 4: verify interface satisfaction (structural typing)
│
└── borrow checker
    ├── track loans per variable: None | Read(spans) | Write(span)
    ├── track moved variables (use after move = error)
    └── rules: no write during reads, no multiple writes, no move then use
```

### Type Compatibility

```
types_compatible(found, expected):
  exact match           → ✅
  impl Trait either side → ✅  (M9.6 — opaque return accepts any concrete type)
  both numeric          → ✅  (Int8 ↔ Int64, etc.)
  wildcard "_"          → ✅  (unresolved generic placeholder)
  same struct/enum name → ✅
  type_map substitution → recurse with substituted types
  otherwise             → ❌  type error
```

---

## 5. Codegen

```
IrEmitter (M4.1: 5 sub-contexts)
├── config      target_triple, check_contracts, strict_mode, hot_reload
├── types        struct layouts, function signatures, interface registry
├── fctx         current function state (locals, params, return type, ensures)
├── mono         monomorphisation state (generic_fn_decls, type_map)
└── local        variable classification, loop stack, module globals

compile_program(program)
├── emit module header        ; XIOM v0.50.0 LLVM IR
├── emit struct types         %struct.Point = type { double, double }
├── emit string constants     @str.0 = private constant [6 x i8] c"hello\00"
├── emit function declarations (forward decls for mutual recursion)
├── for each function:
│   ├── emit function signature   define i64 @add(i64 %a, i64 %b)
│   ├── emit allocas              %x = alloca i64
│   ├── emit contract guards      requires → br cond, body, @llvm.trap
│   ├── compile body
│   │   ├── compile_stmt()        let/var, return, if, match, while, for, assign
│   │   └── compile_expr()        literals, binary, call, field, index, struct
│   └── emit return
├── emit monomorphised functions (generics — two-pass register + specialize)
└── emit module globals
```

### LLVM Type Mapping

```
XIOM         →  LLVM IR
─────────────────────────
Int          →  i64
Float64      →  double
Bool         →  i1 (i64 in structs)
Str          →  i8*
Char         →  i32
*T, &T       →  i64          (pointers lowered to integer)
Option[T]    →  { i64, i64 }  (discriminant, value)
Result[T,E]  →  { i64, i64, i64 }
Vec[T]       →  { i8*, i64, i64, i64 }  (data, len, cap, elem_size)
struct S     →  %struct.S { ... }
enum E       →  { i64, i64 }  (discriminant, payload union)
[N]T         →  { i64, [N x elem_type] }  (length + fixed array)
```

### Contract Emission

```
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures:  result * b == a
{ return a / b; }
```

```
define double @divide(double %a, double %b) {
  %req = fcmp une double %b, 0.0          ; requires check
  br i1 %req, label %body, label %trap
body:
  %result = fdiv double %a, %b
  %ens = fmul double %result, %b           ; ensures check
  %ok = fcmp oeq double %ens, %a
  br i1 %ok, label %return, label %trap
trap:
  call void @llvm.trap()
  unreachable
return:
  ret double %result
}
```

---

## 6. Execution Modes

### AOT (Ahead-of-Time)

```
Source → Lex → Parse → Check → Codegen → LLVM IR → clang → binary

xiom main.xi -o app.exe          Standard build
xiom main.xi --release            LLVM -O3 optimization
xiom main.xi --target wasm        WASM output
xiom main.xi --shared             Shared library (.dll/.so)
xiom --check main.xi              Type-check only, no binary
xiom --emit-ir main.xi            Print IR, no linking
```

### Scripting Mode (`xiom run`)

```
Source
  │
  ├── Shebang strip:     #!/usr/bin/env xiom → skip line
  ├── Implicit main:     io.println("hi") → fn main() { io.println("hi") }
  ├── Auto-import:       adds use xiom.io; if missing
  └── Declarations:      type/enum/fn/use stay at top level outside main()

  Then: compile → execute

Methods of execution:
  Default:   compile → temp .exe → run as subprocess
  --jit:     compile → .dll → libloading::Library::new() → main() in-process
  --watch:   poll file mtime → recompile + rerun on change
  Cache:     content-hash → ~/.xiom/jit/ → instant re-run (100 MB LRU eviction)
```

```
┌─────────────────────────────────────────────────────┐
│                  xiom run flow                       │
│                                                      │
│  xiom run -e "code"    inline expression            │
│  xiom run -            read from stdin               │
│  xiom run file.xi      execute script file           │
│  xiom run --watch      auto re-run on file change    │
│  xiom run --jit        in-process DLL execution      │
│                                                      │
│  All paths:                                          │
│  1. Apply shebang + implicit main + auto-imports     │
│  2. Compile (same pipeline as AOT)                   │
│  3. Execute (subprocess or in-process JIT)           │
│  4. Cache compiled binary (content-hash key)         │
└─────────────────────────────────────────────────────┘
```

### Standalone (`xiom --standalone`)

```
Script → wrap → compile --release → standalone production binary

xiom --standalone myscript.xi -o mytool.exe
xiom --standalone --scaffold myscript.xi    also creates project structure:
  myscript/
    src/main.xi     canonicalized script
    package.xi      project manifest
```

### Interactive REPL (`xiom repl`)

```
┌──────────────────────────────────────────┐
│              xiom repl                    │
│                                           │
│  xiom> var x = 42                         │
│  xiom> io.println(x.to_str())             │
│  42                                       │
│  xiom> :vars                              │
│    var x = 42                             │
│  xiom> :reset                             │
│    State cleared.                         │
│  xiom> :list io                           │
│    io.println, io.print, io.read_line...  │  (M13)
│  xiom> :quit                              │
│                                           │
│  State persistence: let/var declarations   │
│  accumulate across lines. Each line is    │
│  compiled as a standalone script with     │
│  accumulated state prepended.             │
└──────────────────────────────────────────┘
```

---

## 7. JIT Architecture

```
xiom run --jit script.xi
        │
        ▼
┌───────────────────┐
│  Implicit main    │
│  + auto-imports   │
└───────┬───────────┘
        ▼
┌───────────────────┐
│  Compile to .dll  │   clang -shared → _jit.dll
│  dllexport @main  │   IR post-processing adds dllexport
└───────┬───────────┘
        ▼
┌───────────────────┐
│  libloading       │   Library::new("_jit.dll")
│  get @main        │   Symbol<unsafe extern "C" fn() -> i64>
│  call main()      │   Returns exit code in-process
└───────┬───────────┘
        ▼
    exit code

Memory: no .exe artifact, DLL loaded and executed entirely in RAM.
AOT parity: same LLVM pipeline, same IR, same clang — identical to AOT binary.
```

---

## 8. Hot Reload & Watch Mode

```
┌──────────────────────────────────────────────────────┐
│              HOT RELOAD (--hot-reload)                 │
│                                                       │
│  Compile-time:                                        │
│  ├── pub functions → thunk table                     │
│  ├── emit @xiom_hot_get_ptr dispatchers               │
│  └── generate export manifest (.hot.json)             │
│                                                       │
│  Runtime:                                             │
│  ├── C runtime monitors file changes                  │
│  ├── On change: recompile delta → reload DLL          │
│  ├── @xiom_hot_get_ptr resolves new function pointers │
│  └── Running state preserved (globals saved/restored) │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│              WATCH MODE (xiom run --watch)             │
│                                                       │
│  loop:                                                │
│    sleep 500ms                                        │
│    check file mtime                                   │
│    if changed:                                        │
│      sleep 200ms (debounce)                          │
│      recompile                                        │
│      rerun                                            │
│    goto loop                                          │
└──────────────────────────────────────────────────────┘
```

---

## 9. LSP Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     VS Code / Editor                      │
│                          │  JSON-RPC                     │
└──────────────────────────┼──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│                      xiom-lsp                             │
│                                                           │
│  ┌────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │ transport  │    │   handlers   │    │   resolver   │ │
│  │            │    │              │    │              │ │
│  │ stdin read │───▶│ initialize   │    │ type lookup  │ │
│  │ Content-   │    │ didOpen      │    │ field access │ │
│  │ Length     │    │ didChange    │    │ fn signature │ │
│  │ framing    │    │ didClose     │    │ symbol table │ │
│  │            │    │              │    │ module tree  │ │
│  │ stdout     │◀───│ hover        │    │              │ │
│  │ write      │    │ completion   │    └──────────────┘ │
│  └────────────┘    │ definition   │                      │
│                    │ signatureHelp│    ┌──────────────┐ │
│  ┌────────────┐    │ docSymbol    │    │   backend    │ │
│  │ diagnostics│    │ references   │    │              │ │
│  │            │    │ rename       │    │ documents    │ │
│  │ parse err  │    │ semanticToken│    │ HashMap      │ │
│  │ type err   │    │ codeAction   │    │ publish      │ │
│  │ borrow err │    │ workspaceSym │    │ diagnostics  │ │
│  └────────────┘    └──────────────┘    └──────────────┘ │
│                                                           │
│  ┌────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │  symbols   │    │semantic_tokens│   │      ai      │ │
│  │            │    │              │    │              │ │
│  │ doc symbol │    │ keyword=0    │    │ .xiom_ai.json│ │
│  │ workspace  │    │ type=1       │    │ hover hint   │ │
│  │ definition │    │ function=2   │    │ error insight │ │
│  └────────────┘    │ variable=3   │    └──────────────┘ │
│                    └──────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

---

## 10. MCP Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   AI Agent / Client                       │
│                          │  JSON-RPC                     │
└──────────────────────────┼──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│                      xiom-mcp                             │
│                                                           │
│  Tools:                                                   │
│  ├── compile_and_analyze     source → diagnostics + IR   │
│  ├── compile_and_fix         source → errors + fix hints │
│  ├── check_xiom_syntax       fast syntax-only validation │
│  ├── format_xiom_code        canonical formatting        │
│  ├── explain_error_code      detailed error reference    │
│  ├── get_contract_signature  fn contracts lookup         │
│  ├── verify_contracts        Z3 SMT proof check          │
│  ├── audit_safety_sandbox    unsafe block audit          │
│  ├── hot_reload_watch        trigger hot recompilation   │
│  └── xiom_stdlib_reference   stdlib API lookup            │
│                                                           │
│  Knowledge:                                               │
│  ├── L_DEBUGGING             debugger commands           │
│  ├── L_ERRORS                error code reference        │
│  ├── L_TOOLCHAIN             build/compile flags         │
│  ├── L_PACKAGING             ecosystem packages          │
│  ├── L_CI_CD                 GitHub Actions integration  │
│  └── L_PUBLISH               registry publishing         │
│                                                           │
│  Workflows:                                               │
│  ├── W_BUILD                 compile + package           │
│  ├── W_DEBUG                 breakpoints + stepping      │
│  ├── W_SCRIPT                scripting + JIT usage       │
│  ├── W_PACKAGE               package management          │
│  └── W_HOTRELOAD             hot reload setup            │
└─────────────────────────────────────────────────────────┘
```

---

## 11. Playground Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Browser (Monaco Editor)                 │
│                                                           │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐ │
│  │  Landing     │    │   Lessons    │    │  Playground │ │
│  │  page        │    │   (372)      │    │  (free code)│ │
│  └─────────────┘    └──────────────┘    └─────────────┘ │
│                                                           │
│  Tabs: Tokens | LLVM IR | Diagnostics | Contracts | Run  │
└──────────────────────────┬───────────────────────────────┘
                           │ HTTP
┌──────────────────────────▼───────────────────────────────┐
│               Playground Server (Node.js)                  │
│                                                           │
│  Endpoints:                                               │
│  ├── POST /api/compile      lex → parse → check → IR     │
│  ├── POST /api/format       xiom-fmt                     │
│  ├── GET  /api/lessons      lesson catalog               │
│  └── GET  /                 static files (HTML/CSS/JS)   │
│                                                           │
│  Compile path:                                            │
│  1. xiom --emit-tokens → token stream                    │
│  2. xiom --check --check-only → diagnostics              │
│  3. xiom --emit-ir --diagnostics-json → IR + errors     │
│  4. xiom run → execute and capture output                │
│                                                           │
│  WASM fallback: xiom-wasm (browser-side, IR only)        │
└──────────────────────────────────────────────────────────┘
```

---

## 12. Crash Recovery & Diagnostics

```
Error Flow:
  Source → Lex → Parse → Check → Codegen
              │       │        │         │
              ▼       ▼        ▼         ▼
           L001    P001     T001      C001
         (lexer) (parser) (checker) (codegen)

Each diagnostic carries:
  { code: "T001", severity: Error,
    message: "Type mismatch: expected Str, found Int",
    span: { file: "main.xi", line: 4, col: 12 },
    suggestion: "Consider using .to_str() to convert Int to Str" }

Error recovery:
  Lexer:   continues past bad characters, marks as Error token
  Parser:  first error terminates (single-error mode)
           → planned M13: collect up to 100 errors before abort
  Checker: continues after type errors (uses Error type as placeholder)
  Codegen: aborts on Error-typed expressions (unrecoverable)
```

---

## 13. Crate Dependency Graph

```
                          xiom
                       (CLI + lib)
                      /    |    |    \
                     /     |    |     \
              xiom-ast  xiom-lexer  xiom-parser  xiom-check  xiom-codegen
                 |         |            |             |            |
                 └─────────┴────────────┴─────────────┴────────────┘
                                        |
                          xiom-graph  xiom-verify  xiom-display  xiom-fmt
                                        |
                    xiom-lsp  xiom-mcp  xiom-pkg  xiom-dbg
                    xiom-doc  xiom-ffigen  xiom-wasm
```

---

## Appendix: XIOM v0.50.0 — All Compilation Paths Reference

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMPLETE FLOW MAP                             │
│                                                                  │
│  BUILD:                                                          │
│    xiom file.xi -o app          → AOT binary                    │
│    xiom file.xi --release       → optimized binary              │
│    xiom file.xi --target wasm   → WASM .wasm                    │
│    xiom file.xi --shared        → shared library .dll/.so       │
│                                                                  │
│  CHECK:                                                          │
│    xiom --check file.xi         → type-check only               │
│    xiom --emit-ir file.xi       → print LLVM IR                 │
│    xiom --emit-tokens file.xi   → print token stream            │
│                                                                  │
│  SCRIPT:                                                         │
│    xiom run file.xi             → execute script                │
│    xiom run -e "code"           → inline expression             │
│    xiom run -                   → stdin script                  │
│    xiom run --watch file.xi     → watch + re-run                │
│    xiom run --jit file.xi       → in-process DLL JIT            │
│                                                                  │
│  TOOLS:                                                          │
│    xiom --standalone file -o exe → script-to-binary             │
│    xiom repl                    → interactive shell             │
│    xiom pkg install <name>      → install package               │
│    xiom pkg search <query>      → search registry               │
│    xiom doctor                  → check toolchain               │
│    xiom clean                   → remove build artifacts        │
│    xiom clean --cache           → clear JIT cache               │
│                                                                  │
│  ADVANCED:                                                       │
│    xiom --verify file.xi        → Z3 contract proof             │
│    xiom --no-contracts          → disable runtime guards        │
│    xiom --sanitize=address      → enable ASan                   │
│    xiom --hot-reload            → hot-reload manifest           │
│    xiom --explain T001          → error code reference          │
└─────────────────────────────────────────────────────────────────┘
```
