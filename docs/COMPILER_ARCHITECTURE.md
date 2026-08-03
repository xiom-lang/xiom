# XIOM Compiler Architecture — v0.56

How the XIOM compiler works — pipeline, stages, data structures, execution modes, and safety features.

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
               │   v0.55: ::, asm, defer     │
               └──────────────┬──────────────┘
                              │
               ┌──────────────▼──────────────┐
               │         PARSER              │
               │   token → AST (Program)     │
               │   LL(1) recursive descent   │
               │   v0.54: const { }, turbofish│
               │   v0.55: asm(), defer, !    │
               └──────────────┬──────────────┘
                              │
               ┌──────────────▼──────────────┐
               │         CHECKER             │
               │   type inference            │
               │   name resolution           │
               │   interface satisfaction    │
               │   borrow checking (E001)    │
               │   v0.54: match exhaustiveness│
               │   v0.55: Send/Sync, Never  │
               └──────────────┬──────────────┘
                              │
               ┌──────────────▼──────────────┐
               │     CTFE (Compile-Time Eval) │
               │   Phase A: const expressions │
               │   Phase B: pure function VM │
               │   v0.54-v0.55: full CTFE    │
               └──────────────┬──────────────┘
                              │
               ┌──────────────▼──────────────┐
               │         CODEGEN             │
               │   AST → LLVM IR text        │
               │   contract guards emitted   │
               │   monomorphisation          │
               │   v0.54: overflow/bounds    │
               │   v0.55: asm, spawn, defer  │
               └──────────────┬──────────────┘
                              │
               ┌──────────────┼──────────────┐
               │              │              │
               ▼              ▼              ▼
            Native         WASM         JIT (DLL)
           clang →      clang →       clang -shared →
          .exe/.out     .wasm      libloading call
             │
        ┌────┴────┐
        │  --lto  │ v0.56: ThinLTO
        │  --debug│ v0.56: DWARF/PDB
        └─────────┘
```

---

## 2. Lexer — v0.55 Extensions

```
New tokens (v0.54-v0.55):
  ColonColon  —  ::  (turbofish: expr::<T>(args))
  Asm         —  asm keyword
  Defer       —  defer keyword

TokenKind enum: 55+ variants including all operators, keywords, literals.
```

---

## 3. Parser — v0.55 Extensions

```
parse_expr() additions:
  ├── ColonColon handler:
  │     ├── ::<Type>(args) → GenericCall  (turbofish)
  │     └── ::method       → Field access (static method)
  │
  └── ConstBlock: const { expr } → Expr::ConstBlock

parse_stmt() additions:
  ├── TokenKind::Asm   → parse_asm_stmt()
  │     └── Stmt::Asm(AsmBlock { template, outputs, inputs, clobbers })
  ├── TokenKind::Defer → parse_defer_stmt()
  │     └── Stmt::Defer(Block)
  └── TokenKind::Never (!) → Type::Never in return position

parse_type_base() additions:
  └── TokenKind::Bang → Type::Never
```

### AST — v0.55 Extensions

```
Expr:
  ├── ConstBlock(Box<Expr>)     — const { expr }
  ├── GenericCall(func, Type, args, span) — turbofish: align_of::<Int>()
  │
Type:
  └── Never                      — ! (bottom type)

Stmt:
  ├── Asm(AsmBlock)             — asm("nop" ::: "rax")
  └── Defer(Block, Span)        — defer { cleanup() }

AsmBlock:
  ├── template: String           — assembly template string
  ├── outputs: Vec<(String, Ident)>   — "=r"(var)
  ├── inputs:  Vec<(String, Expr)>   — "r"(expr)
  └── clobbers: Vec<String>     — ["rax", "memory"]

CheckedType:
  └── Never                      — bottom type (compatible with everything)
```

---

## 4. Checker — v0.55 Extensions

```
Type compatibility (types_compatible):
  + Never is compatible with everything (bottom type)
  + Generic type params (T, K, V) compatible with any type
  + Wildcard "_" compatible with everything

Match exhaustiveness (S2):
  + pattern_covers_variant() checks Option/Result/enum completeness
  + --strict-exhaustive flag promotes warnings to errors

Thread safety:
  + interface Send { }      — marker trait, auto-derived
  + interface Sync { }      — marker trait, auto-derived
  + All primitives are Send + Sync

Builtins registered:
  + sizeof::<T>() → Int
  + align_of::<T>() → Int
  + type_id::<T>() → Int
  + field_offset::<T>(Str) → Int
  + is_signed::<T>() → Bool
```

---

## 5. CTFE — Compile-Time Function Evaluation

```
Phase A: Const Expression Evaluator (crates/xiom-codegen/src/expr.rs)
  ├── evaluate_const_init() — recursive const folder
  ├── Arithmetic, comparison, boolean, unary, bitwise ops
  ├── Builtins: sizeof, align_of, type_id, field_offset, is_signed
  ├── if/else folding: Bool condition → select branch
  ├── match folding: pattern matching with variable bindings
  ├── Const variable references: resolve through const table
  └── String comparison: ==, != at compile time

Phase B: Full CTFE Interpreter (crates/xiom-ctfe/src/lib.rs)
  ├── CtfeEngine — tree-walking interpreter
  ├── Recursive function evaluation (depth limit 1000)
  ├── While loops (step limit 100K)
  ├── Pattern matching with bindings (Some(v) => v)
  ├── Arena allocator (256MB bounded)
  └── Builtins: str_len, str_concat, int_to_string
```

---

## 6. Codegen — v0.55 Extensions

```
IrEmitter (M4.1: 6 sub-contexts)
├── config      + v0.55: strict_exhaustive, overflow_checks
├── types       + v0.54: SyncRegistry (Arc<RwLock<HashMap>>)
├── fctx         current function state
├── mono         monomorphisation state
├── local        + v0.55: spawn_counter, spawn_declared
└── ctfe         + v0.54: RefCell<CtfeEngine>

compile_program():
  + v0.55: declare @xiom_thread_spawn, @xiom_channel_*, @xiom_threadpool_*
  + v0.56: declare @xiom_threadpool_init/spawn
  + v0.56: --lto flag → -flto=thin

compile_stmt_impl():
  + Stmt::Asm     → call void asm sideeffect "..." "~{...}"()
  + Stmt::Spawn   → compile body as @_xiom_spawn_N, call xiom_thread_spawn
  + Stmt::Defer   → emit cleanup block inline

LLVM Type Mapping:
  + Never (!)     → i64 (return register, unreachable after call)
```

### Inline ASM Codegen

```
Source:
  asm("nop");
  asm("mov $0, 42" : "=r"(result) : "r"(input));

Generated IR:
  call void asm sideeffect "nop", "~{dirflag},~{fpsr},~{flags}"()
  call void asm sideeffect "mov $0, 42", "~{dirflag},~{fpsr},~{flags}"()
```

### Spawn Codegen

```
Source:
  spawn { heavy_work(); }

Generated IR:
  ; Spawn wrapper function (emitted via deferred_closure_defs)
  define void @_xiom_spawn_0(i8* %_xiom_spawn_arg) {
  entry:
    ; body IR
    ret void
  }

  ; In calling function:
  %handle = call i64 @xiom_thread_spawn(ptr @_xiom_spawn_0, ptr null)
```

---

## 7. Execution Modes — v0.56

### AOT (Ahead-of-Time)

```
xiom file.xi -o app.exe              Standard build
xiom file.xi --release                LLVM -O3 optimization
xiom file.xi --lto                    ThinLTO (v0.56)
xiom file.xi --debug / -g             DWARF/PDB debug info (v0.56)
xiom file.xi --target wasm            WASM output
xiom file.xi --shared                 Shared library (.dll/.so)
xiom --check file.xi                  Type-check only, no binary
xiom --emit-ir file.xi                Print IR, no linking
xiom --parallel                       Rayon-based parallel parse (v0.54)
xiom --cache                          Binary cache (v0.54)
xiom --overflow-checks                Integer overflow traps (v0.54)
xiom --strict-exhaustive              Non-exhaustive match → error (v0.54)
```

### Scripting Mode (`xiom run`)

```
xiom run file.xi              Execute script
xiom run -e "code"            Inline expression
xiom run -                    stdin script
xiom run --watch file.xi      Watch + re-run
xiom run --jit file.xi        In-process DLL JIT (v0.55)
xiom run --jit --lazy file.xi JIT with incremental cache (v0.56)
xiom run --cache              Binary cache for instant re-run (v0.54)
xiom run --no-cache           Disable caching
```

### JIT Architecture (v0.55)

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
│  via xiom-jit     │   Pre-compiled libxiom_runtime.dll linked
└───────┬───────────┘
        ▼
┌───────────────────┐
│  libloading       │   Library::new("_jit.dll")
│  get @main        │   Symbol<unsafe extern "C" fn() -> i64>
│  call main()      │   Returns exit code in-process
└───────┬───────────┘
        ▼
    exit code

Cache: SHA-256 source hash → ~/.xiom/jit/<hash>
      100 MB LRU eviction. Warm: 5ms, Cold: ~150ms.
```

---

## 8. C Runtime — v0.55+v0.56

```
stdlib/runtime/
├── xiom_runtime.c       (4400+ lines)
│   ├── threads:    xiom_thread_create/join/detach, mutex, condvar
│   ├── spawn:      xiom_thread_spawn/join/detach, 256-slot table
│   ├── channel:    xiom_channel_create/send/recv/try_recv/close (v0.55)
│   ├── threadpool: xiom_threadpool_init/spawn/shutdown (v0.56)
│   ├── filesystem: xiom_read_file/write_file, stat, dir listing
│   ├── crypto:     AES-NI, SHA-NI, SHA-256 software, Ed25519
│   ├── memory:     malloc/free, memcpy/memset/memcmp
│   └── platform:   Windows (CRITICAL_SECTION, CONDITION_VARIABLE)
│                   Linux (pthread_mutex_t, pthread_cond_t)
├── async_runtime.c
├── simd_runtime.c
├── sha256_sw.c
└── xiom_hot_reload.c

Pre-compiled: libxiom_runtime.dll (via xiom build-runtime)
Module declares: inserted at IR module level by codegen prologue.
Auto-stub suppression: 7 runtime functions excluded from dead-code stubs.
```

---

## 9. Crate Dependency Graph — v0.55

```
                        xiom
                     (CLI + lib)
                    /    |    |    \
                   /     |    |     \
            xiom-ast  xiom-lexer  xiom-parser  xiom-check  xiom-codegen
               |         |            |             |            |
               └─────────┴────────────┴─────────────┴────────────┘
                                      │
                        xiom-graph  xiom-verify  xiom-display  xiom-fmt
                                      │
                  xiom-lsp  xiom-mcp  xiom-pkg  xiom-dbg
                  xiom-doc  xiom-ffigen  xiom-wasm
                  xiom-ctfe (v0.54)  xiom-jit (v0.55)
```

---

## 10. Complete Flow Map — v0.56

```
BUILD:
  xiom file.xi -o app              → AOT binary
  xiom file.xi --release           → optimized binary
  xiom file.xi --lto               → ThinLTO optimized (v0.56)
  xiom file.xi --debug             → with DWARF/PDB (v0.56)
  xiom file.xi --target wasm       → WASM .wasm
  xiom file.xi --shared            → shared library .dll/.so

CHECK:
  xiom --check file.xi             → type-check only
  xiom --emit-ir file.xi           → print LLVM IR
  xiom --emit-tokens file.xi       → print token stream

SCRIPT:
  xiom run file.xi                 → execute script
  xiom run -e "code"               → inline expression
  xiom run -                       → stdin script
  xiom run --watch file.xi         → watch + re-run
  xiom run --jit file.xi           → in-process DLL JIT (v0.55)
  xiom run --cache                 → binary cache (v0.54)

SAFETY:
  xiom --overflow-checks           → integer overflow traps
  xiom --strict-exhaustive         → non-exhaustive match → error
  xiom --sanitize=address          → enable ASan
  xiom --sanitize=undefined        → enable UBSan
  xiom --sanitize=thread           → enable TSan
  xiom --no-contracts              → disable runtime guards
  xiom --verify file.xi            → Z3 contract proof

TOOLS:
  xiom --standalone file -o exe    → script-to-binary
  xiom repl                        → interactive shell
  xiom pkg install <name>          → install package
  xiom build-runtime               → build libxiom_runtime.dll (v0.55)
  xiom doctor                      → check toolchain
  xiom clean                       → remove build artifacts
  xiom clean --cache               → clear JIT cache
  xiom --explain T001              → error code reference
```

---

**Status:** Updated to reflect v0.56 implementation state. All new crates, pipeline stages, flags, and features documented.
