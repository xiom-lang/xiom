# XIOM Compiler Architecture — v0.57

How the XIOM compiler works — pipeline, stages, data structures, execution modes, safety features, and the Unsafe Confinement subsystem.

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
               │   v0.57: fn attributes →    │
               │    parse_fn_decl (unsafe_   │
               │    no_retry / unsafe_direct)│
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
               │   v0.57: Unsafe Confinement │
               │    gates (T002/T003/T005/   │
               │    T006/T007, block-only)   │
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
               │   v0.57: unsafe blocks →    │
               │    __unsafe_block_N(ctx) +  │
               │    SEH trampoline calls;    │
               │    guard-arena allocs;      │
               │    Copy-Out Str tails       │
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
              │
         ┌────▼──────────────────────────────┐
         │  v0.57 RUNTIME (linked always)    │
         │  xiom_trampoline_call (SEH/sigsetjmp)│
         │  xiom_guard_heap_enter/exit/alloc │
         │  xiom_guard_page_arm/disarm       │
         │  xiom_guard_copy_str / realloc    │
         └───────────────────────────────────┘
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

## 3. Unsafe Confinement — v0.57 (P1–P8 + T006)

### 3.1 Gates (checker — `crates/xiom-check/src/lib.rs`)

```
Lexical (a):   `unsafe` applies STRICTLY to the block. `unsafe fn/module/struct/impl`
               → parse hard error with clear diagnostic.
T002 (b):      extern "C" call from safe code (unsafe_depth == 0) → hard error.
               Exempt: fns declaring requires/ensures contracts (safe-wrapper pattern).
T003 (c):      safe fn returning raw pointer (*T) → hard error, unless body contains
               unsafe (unsafe-internal helper exemption, block_contains_unsafe).
T007 (d):      fn whose ENTIRE body is one unsafe block must declare `requires`.
T005 (i):      raw ptr / &T / fn type / struct containing them cannot be an unsafe
               block's tail (enforce_fn_unsafe_tail at fn boundary).
T006 (j):      extern-returned *T inside a confined block must convert to an owned XIOM
               type before the tail (ffi.safe_ptr_from_raw / box_from_ptr /
               vec_from_ptr_with_free / str_from_ptr_owned). Exempt: fns that THEMSELVES
               return a raw pointer (allocator pattern — caller owns it).

Checker state (per fn):
  unsafe_depth: Int          — block nesting; > 0 while inside unsafe { }
  pending_extern_ptrs        — extern calls returning *T inside the block
  converted_ffi_ptrs         — locals passed through a registered conversion fn
  current_fn_has_requires    — for T007
```

### 3.2 Trap lowering (codegen — `crates/xiom-codegen/src/expr.rs`, Expr::Unsafe)

The original inline VEH approach (xiom_trap_enter + RtlCaptureContext/
RtlRestoreContext) was BROKEN: RtlRestoreContext resumed into xiom_trap_enter's
already-popped/reused frame, so the epilogue `ret` popped a stale return address
and re-entered the faulting block (infinite AV→restore→AV loop — the P5 hang).

Canonical v0.57 lowering (plan §2.7):

```
Expr::Unsafe(block)  ──►  standalone fn  define i64 @__unsafe_block_N(i8* %ctx)
                          (globally-unique id via unsafe_block_counter;
                           NOT alwaysinline)
                              │
                              ▼
                          at the call site:
                            %fault = call i64 @xiom_trampoline_call(
                                        ptrtoint(@__unsafe_block_N), %ctx_i8)
                            branch %fault != 0 → confined_fault (ret 0/zero)
                                            else → confined_normal (recover result
                                                   via xiom_trampoline_get_result)

Block fn body:
  entry:
    %__ctx_ptr = bitcast i8* %ctx_raw to %struct.__unsafe_ctx_N*   ; captures
    ; load capture POINTERS (write-back semantics — assignments inside the block
    ;   propagate to the enclosing scope, e.g. pad_str = str_concat(...))
    call void @xiom_guard_heap_enter()     ; req (e): guard arena
    call void @xiom_guard_page_arm()       ; req (f): stack guard page
    ... block statements (allocations route to @xiom_guard_alloc) ...
    ; Copy-Out (req i): Str tail → @xiom_guard_copy_str BEFORE arena reset
    call void @xiom_guard_heap_exit()
    call void @xiom_guard_page_disarm()
    ret i64 <val_to_i64(tail)>

Nested unsafe blocks (already inside an unsafe-block fn) compile as PLAIN blocks
(no second trampoline — the outer SEH checkpoint covers them; §2.13).
```

### 3.3 Retry (codegen + runtime — req h, P6)

```
xiom_trampoline_call loop (initial + ONE retry):
  attempt 1:  __try { result = fn(ctx); }
              __except → map code (1=SIGSEGV, 2=SIGILL, 3=SIGFPE,
                                    4=STACK_OVERFLOW, 5=GUARD_PAGE, 6=other)
  if transient && xiom_trampoline_allow_retry && !retried:
      xiom_guard_heap_exit(); xiom_guard_page_disarm();   ; fresh memory slot
      retried = 1; continue                               ; re-run fn(ctx)
  if ok:   xiom_trampoline_last_result = result; return 0
  else:    return fault_code                              ; recoverable zero

#[unsafe_no_retry] (fn attribute) → codegen emits set_allow_retry(0) before the call.
```

### 3.4 `#[unsafe_direct]` (P7 — trusted escape hatch)

```
Parser: leading '#' at top level routes to parse_fn_decl → fn attributes consumed.
Codegen: fctx.unsafe_direct set from the attribute. RESTRICTED to stdlib/selfhost
         sources (config.source_file check); user code needs --enable-unsafe-direct.
Direct blocks compile as PLAIN unsafe (no trampoline/arena/guard page) and are
counted against config.unsafe_direct_cap (default 64 — exceeding = hard error).
```

### 3.5 Guard arena (runtime — req e, P3)

```
Per-thread TLS:  xiom_guard_arena { active, cur_off, cur_slab, slabs[] }
Slabs: VirtualAlloc (win) / mmap (posix), 64 KB, chained.
xiom_guard_alloc(size)  → bump-allocate inside current slab (interior pointers —
                          NEVER realloc'd; growth uses xiom_guard_realloc).
xiom_guard_heap_enter() → active++;  xiom_guard_heap_exit() → discard ALL slabs.
Vec growth inside confined blocks routes to @xiom_guard_realloc (arena-aware copy;
plain realloc on VirtualAlloc interior pointers is invalid — this FIXED the
guard-heap Vec-growth crash/hang).
```

### 3.6 Stack guard page (runtime — req f, P4)

```
xiom_guard_page_arm():   install a PAGE_GUARD (win) / PROT_NONE (posix) red-zone
                         region per thread; overflow faults at the guard page
                         (0xC000001D) BEFORE adjacent memory is written.
xiom_guard_page_disarm(): restore the region.
```

---

## 4. Parser — v0.55 Extensions (v0.57 attributes)
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
  ├── GenericCall(func, Vec<Type>, args, span) — turbofish: align_of::<Int>()
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

## 5. Checker — v0.55 Extensions

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

Unsafe Confinement gates (v0.57 — see §3.1 for full rules):
  + unsafe_depth tracking; extern gate (T002), signature gate (T003),
    whole-body-unsafe requires gate (T007), zero-escape tail gate (T005),
    FFI ownership gate (T006), block-only unsafe parse diagnostic
  + fn attributes consumed: #[unsafe_no_retry] → unsafe_allow_retry=false,
    #[unsafe_direct] → fctx.unsafe_direct (stdlib/selfhost or --enable-unsafe-direct)
```

---

## 6. CTFE — Compile-Time Function Evaluation

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

Scaling note (post-selfhost, v0.58+): the CTFE engine lives on the IrEmitter as
RefCell<CtfeEngine>. For parallel codegen (128 workers) the result CACHE moves to
a thread-safe CtfeCache = RwLock<HashMap<u64, CtfeValue>> in the SyncRegistry —
compute-once per constant, minimal lock contention (docs/CTFE_PLAN.md §6, at
Scaling Phase 5 — NOT before selfhost).
```

---

## 7. Codegen — v0.55 Extensions (v0.57 Unsafe Confinement)

```
IrEmitter (M4.1: 6 sub-contexts)
├── config      + v0.55: strict_exhaustive, overflow_checks
│               + v0.57: enable_unsafe_direct, unsafe_direct_cap
├── types       + v0.54: SyncRegistry (Arc<RwLock<HashMap>>)
├── fctx        + v0.57: unsafe_allow_retry, unsafe_direct, is_never_return
├── mono         monomorphisation state
├── local        + v0.55: spawn_counter, spawn_declared
│                + v0.57: deferred_closure_defs (unsafe-block fns flushed here)
└── ctfe         + v0.54: RefCell<CtfeEngine>

Emitter-wide (v0.57):
  unsafe_block_counter   — globally-unique id for __unsafe_block_N (NOT reset per
                           function; tmp/block counters reset per fn, so a separate
                           counter prevents symbol collisions across enclosing fns)
  unsafe_direct_count    — audited count of #[unsafe_direct] blocks vs the cap
  in_unsafe_block_fn     — 1 while compiling an unsafe-block fn body

compile_program():
  + v0.55: declare @xiom_thread_spawn, @xiom_channel_*, @xiom_threadpool_*
  + v0.56: declare @xiom_threadpool_init/spawn
  + v0.56: --lto flag → -flto=thin
  + v0.57: declare @xiom_trampoline_* / @xiom_guard_* (see §9 declares)

compile_stmt_impl():
  + Stmt::Asm     → call void asm sideeffect "..." "~{...}"()
  + Stmt::Spawn   → R2: capture analysis + env forwarding + compile body as @_xiom_spawn_N
  + Stmt::Defer   → emit cleanup block inline

Expr::Unsafe (v0.57 — canonical trap lowering, see §3.2):
  + standalone i64 @__unsafe_block_N(i8* %ctx) via deferred_closure_defs
  + call site: xiom_trampoline_call + branch confined_fault / confined_normal
  + pointer captures (ctx fields hold enclosing alloca ADDRESSES — write-back)
  + Str tail Copy-Out (@xiom_guard_copy_str) BEFORE arena reset
  + nested unsafe (in_unsafe_block_fn) → plain block, no second trampoline (§2.13)
  + #[unsafe_direct] (fctx.unsafe_direct) → plain block, no confinement
  + return routing: Stmt::Return inside a block fn emits
    @xiom_trampoline_set_returned() first; call site emits the enclosing return
  + fault path returns a type-correct zero (recoverable indicator)

emit_alloc / emit_realloc (v0.57):
  + guard_heap_depth > 0 → @xiom_guard_alloc / @xiom_guard_realloc (arena)
  + else → @malloc / @realloc

Vec growth (call.rs, v0.57): realloc sites route through emit_realloc so Vec
growth inside confined blocks uses the arena-aware realloc (FIXED a crash/hang
on VirtualAlloc slab interior pointers).

Parallel Codegen (I2 — v0.56):
  + --parallel-codegen flag → rayon::par_iter() across function bodies
  + Each function gets own IrEmitter clone with shared TypeContext
  + Outputs merged in declaration order after all tasks complete
  + Maps to XIOM selfhost pattern: spawn + Channel[T] collect
  + v0.57: generic monomorphisation path ALSO flushes deferred_closure_defs
    (or __unsafe_block_N defs from generic bodies would be dropped)

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

### Spawn Codegen (v0.55 → v0.56 R2)

```
Source (v0.55):
  spawn { heavy_work(); }

Source (v0.56 R2 — move semantics):
  spawn move { heavy_work(); }
  var x = 42;
  spawn move { var result = x + 1; }   // x captured by value

Generated IR (v0.56 with captures):
  ; Spawn wrapper function (emitted via deferred_closure_defs)
  define void @_xiom_spawn_0(i8* %_xiom_spawn_arg) {
  entry:
    ; Unpack captures from env buffer (flat i64 array at offset i*8)
    %cap0_ptr = bitcast i8* %_xiom_spawn_arg to i64*
    %cap0_val = load i64, i64* %cap0_ptr
    %cap0_alloca = alloca i64
    store i64 %cap0_val, i64* %cap0_alloca  ; x is now available as local
    ; body IR
    ret void
  }

  ; In calling function:
  %env = call i8* @malloc(i64 N)          ; allocate env for N captures
  ; Null check + trap if OOM
  %x_val = load i64, i64* %x_alloca
  store i64 %x_val, i64* (bitcast %env to i64*)
  %handle = call i64 @xiom_thread_spawn(ptr @_xiom_spawn_0, ptr %env)
```

---

## 8. Execution Modes — v0.57

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
xiom --enable-unsafe-direct           Allow #[unsafe_direct] in user code (v0.57)
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

## 9. C Runtime — v0.55+v0.57

```
stdlib/runtime/
├── xiom_runtime.c       (5000+ lines)
│   ├── threads:    xiom_thread_create/join/detach, mutex, condvar
│   ├── spawn:      xiom_thread_spawn/join/detach, 256-slot table
│   ├── channel:    xiom_channel_create/send/recv/try_recv/close (v0.55)
│   ├── threadpool: xiom_threadpool_init/spawn/shutdown (v0.56)
│   ├── filesystem: xiom_read_file/write_file, stat, dir listing
│   ├── crypto:     AES-NI, SHA-NI, SHA-256 software, Ed25519
│   ├── memory:     malloc/free, memcpy/memset/memcmp
│   ├── confinement (v0.57):
│   │   ├── xiom_trampoline_call(fn, ctx)    — SEH __try/__except checkpoint
│   │   │     (POSIX: sigsetjmp/siglongjmp); returns 0=ok / 1-6=fault; once-only
│   │   │     transient retry (reset arena + re-arm page + re-run fn)
│   │   ├── xiom_trampoline_get_result/set_returned/was_returned/
│   │   │     was_retried/set_allow_retry    — TLS result + return-routing slots
│   │   ├── xiom_guard_heap_enter/exit/alloc — per-thread 64KB VirtualAlloc/mmap
│   │   │     slab arena; exit discards ALL slabs
│   │   ├── xiom_guard_realloc(old, old_sz, new_sz) — arena-aware grow (copy to
│   │   │     a fresh slab; plain realloc on slab interior ptrs is invalid)
│   │   ├── xiom_guard_copy_str / copy_out   — Copy-Out Str tails to main heap
│   │   ├── xiom_guard_page_arm/disarm       — PAGE_GUARD/PROT_NONE red-zone
│   │   └── fault-injection helpers: xiom_fault_av/ud2/div0/transient/permanent
│   └── platform:   Windows (CRITICAL_SECTION, CONDITION_VARIABLE)
│                   Linux (pthread_mutex_t, pthread_cond_t)
├── async_runtime.c
├── simd_runtime.c
├── sha256_sw.c
└── xiom_hot_reload.c

Pre-compiled: libxiom_runtime.dll (via xiom build-runtime)
Module declares: inserted at IR module level by codegen prologue.
Auto-stub suppression: 7 runtime functions excluded from dead-code stubs.
Confinement declares (codegen prologue, v0.57):
  declare i64 @xiom_trampoline_call(i64, i8*)
  declare i64 @xiom_trampoline_get_result()
  declare void @xiom_trampoline_set_returned()
  declare i64 @xiom_trampoline_was_returned()
  declare i64 @xiom_trampoline_was_retried()
  declare void @xiom_trampoline_set_allow_retry(i64)
  declare i8* @xiom_guard_alloc(i64)
  declare i8* @xiom_guard_realloc(i8*, i64, i64)
  declare i8* @xiom_guard_copy_str(i8*)
  declare void @xiom_guard_heap_enter() / @xiom_guard_heap_exit()
  declare void @xiom_guard_page_arm() / @xiom_guard_page_disarm()
```

---

## 10. Crate Dependency Graph — v0.57

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

## 11. Complete Flow Map — v0.57

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
  xiom --sandbox file.xi           → legacy safety audit (confinement gates are the primary layer)
  xiom --sandbox --sandbox-report=json file.xi  → JSON audit (v0.57: MUST include --sandbox first)
  xiom --enable-unsafe-direct      → allow #[unsafe_direct] in user code (v0.57)

UNSAFE CONFINEMENT (v0.57 — automatic, no flag needed):
  unsafe { ... }                   → confined: guard arena + stack guard + SEH trap
                                     + once-only transient retry + Copy-Out
  #[unsafe_no_retry] fn            → disables the once-only retry
  #[unsafe_direct] fn              → trusted escape hatch (stdlib/selfhost; or
                                     --enable-unsafe-direct for user code)

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

**Status:** Updated to reflect v0.57 implementation state (Unsafe Confinement P1–P8 + T006). All new crates, pipeline stages, flags, gates, and runtime features documented.
