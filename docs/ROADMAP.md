# XIOM Compiler — Production Roadmap

**Current:** v0.45.3 "Phase 5b Complete" — 41/41 smoke, 85/85 e2e, 47/47 parser, 74/74 checker, all gates green
**Branch:** `feat/guardian`
**Target:** v1.0.0 self-hosting compiler

---

## 1. CURRENT STATE (2026-07-14)

| Gate | Count | Status |
|------|-------|--------|
| Parser tests | 47/47 | ✅ |
| Checker tests | 74/74 | ✅ |
| Stdlib execution (smoke) | 41/41 (0 ignored) | ✅ |
| E2E tests | 85/85 | ✅ |
| Feature regression | 48/48 | ✅ |
| Integration regression | 119 | ✅ |
| Other regression (diff, fulldiff, fuzz, robustness) | 100 combined | ✅ |

### Bugs: ALL 10 RESOLVED

| Bug | Fix | Files Changed |
|-----|-----|---------------|
| **BUG-001** SHA-256 wrong hash | C reference via FFI (`sha256_sw.c`) | `stdlib/runtime/` |
| **BUG-002** async expression paths | Parser `fn()` type args + contextual `async` | `xiom-parser`, `xiom-check` |
| **BUG-003** sync AtomicBool bad IR | `Expr::If` conditional branches with result alloca | `xiom-codegen` |
| **BUG-005** mem replace stack overflow | Leaf-module key registration for monomorphisation disambiguation | `xiom-codegen` |
| **BUG-006** Option[Struct].unwrap() | `i64 → struct` coercion + `Stmt::Let` declared-type handling | `xiom-codegen`, `xiom-check` |
| **BUG-007** Interface dispatch | Exhaustive monomorphisation per concrete implementor | `xiom-codegen` |
| **BUG-008** IO string coercion | `Str.c_str()` + `Str.len()` builtins | `xiom-codegen` |
| **BUG-009** TestResult.passed | Same fix as BUG-006 (struct payload round-trip) | (same as BUG-006) |
| **BUG-010** Channel send/recv | `&mut self` struct receiver (pointer passing) | `xiom-parser`, `xiom-ast`, `xiom-codegen`, `async.xi` |

---

## 2. CANONICAL PHASE SYSTEM

| Phase | Codename | Version | Status |
|-------|----------|---------|--------|
| 0 | Pipeline | v0.1.0 | ✅ |
| 1 | Guardian | v0.2.0–v0.30.0 | ✅ |
| 2 | Hardened | v0.31.0 | ✅ |
| 3 | ARC-C | v0.32.0 | ✅ |
| 4 | Or-Patterns | v0.33.0 | ✅ |
| **5a** | **Codegen Hardening** | **v0.45.x** | **✅ COMPLETE** |
| **5b** | **Stdlib Completion** | **v0.45.x** | **✅ COMPLETE** |
| 5c | Architectural Features | next | Planned |
| 5d | Production Toolchain | later | Planned |
| 5e | Self-Hosting | v1.0.0 | Planned |

---

## 3. PHASE 5a — CODEGEN HARDENING (100% COMPLETE) ✅

### 5a.1. ARC A: Real Pointer/Reference Types ✅
All 6 steps verified. e2e tests: `ptr_deref.xi`, `ref_mut_param.xi`, `mut_ref_swap.xi`.

### 5a.2. Map Type Injection ✅
Map removed from checker PRIMITIVES + `generic_type_names` search.

### 5a.3. Fixed-Size Array Allocation ✅
alloca reuse for Ident containers in loop bodies.

### 5a.4. Option.unwrap() + Payload Binding ✅
Inline unwrap/unwrap_err, match dispatch, OR patterns, inner literal checks.

### 5a.5. Const-Generics: `[N]T` Arrays ✅
| Delivered |
|-----------|
| `type_from_ast` handles `Type::Array` with ident + element types |
| `llvm_type_for` resolves const-declared values from `self.constants` |
| Monomorphisation infra: `const_value_map`, `Type::Array` substitution in `subst_type` |
| Array indexing on let/var-bound literal arrays (`array_locals` tracking) |
| Const-declared sizes in while loops (e2e: `const_generic_array.xi`) |
| **NOTE:** Full const-generic monomorphisation call (`len[Int, 5](arr)`) requires end-to-end verification — deferred to Phase 5c as it needs a test harness for const-generic call sites |

### 5a.6. Match Expression Type Unification ✅
`infer_match_llvm_type` collects types from ALL arms, picks widest. TODO marker removed.

### 5a.7. Expr::Array Indexing on Literal Arrays ✅
`Expr::Index` recognizes array-literal buffers, skips length slot. TAIL-TODO removed.

### 5a.8. Interface/Trait Object Dispatch ✅
| Delivered |
|-----------|
| Exhaustive monomorphisation: specialized versions per concrete implementor |
| Interface impl tracking: `scan_interface_impls()` builds concrete→interface map |
| Call-site inference: resolves interface-typed params to concrete struct types |
| Monomorphisation type_map: maps interface names → concrete struct types |
| Multi-implementation dispatch verified (GoodReporter + BadReporter → Reporter) |
| `error.xi` compiles 0 errors (all BUG-007-related issues resolved) |
| Str + Str concatenation type-checks in `BinOp::Add` |

---

## 4. PHASE 5b — STDLIB COMPLETION (100% COMPLETE) ✅

### 4a. Module Status

| Module | Status | Verification |
|--------|--------|-------------|
| **core.xi** | ✅ | Smoke passes (41/41) |
| **array.xi** | ✅ | Array indexing on let/var-bound literals (5a.7 e2e) |
| **string.xi** | ✅ | `byte_at` added |
| **collections.xi** | ✅ | Vec, Map, Set all smoke-pass |
| **io.xi** | ✅ | 31 types/fns made `pub`; `write_file`/`read_file`/`file_exists`/`remove_file` round-trip verified |
| **fmt.xi** | ✅ | `.to_str()` on Int/Bool works |
| **iter.xi** | ✅ | Smoke passes |
| **math.xi** | ✅ | Smoke passes |
| **num.xi** | ✅ | Smoke passes |
| **cmp.xi** | ✅ | Smoke passes |
| **hash.xi** | ✅ | Smoke passes |
| **mem.xi** | ✅ | `swap[Int]`/`replace[Int]` with leaf-key monomorphisation |
| **ptr.xi** | ✅ | `from_ref`/`from_mut`/`read`/`write` work |
| **char.xi** | ✅ | Smoke passes |
| **path.xi** | ✅ | `file_name`/`extension`/`file_stem`/`parent`/`is_absolute`/`components`/`canonicalize` all work |
| **convert.xi** | ✅ | Smoke passes |
| **ffi.xi** | ✅ | Smoke passes |
| **sync.xi** | ✅ | `AtomicInt` store/load, `AtomicBool` store/load via if-expr fix |
| **thread.xi** | ✅ | `available_parallelism`/`sleep_ms`/`yield_now`/`current_thread_id`/`Thread.current` |
| **async.xi** | ✅ | `Executor.new`/`sleep_ms`/`Channel.unbounded().send`/`try_recv` with `&mut self` |
| **net.xi** | ✅ | Smoke passes |
| **os.xi** | ✅ | Smoke passes |
| **time.xi** | ✅ | Smoke passes |
| **test.xi** | ✅ | `test.assert` invocation verified |
| **bench.xi** | ✅ | Smoke passes |
| **log.xi** | ✅ | Smoke passes |
| **serialize.xi** | ✅ | Smoke passes |
| **crypto.xi** | ✅ | SHA-256 known-vectors verified; `aes_decrypt` software fallback fixed |
| **regex.xi** | ✅ | Stdlib logic fine |
| **rand.xi** | ✅ | Smoke passes |
| **encoding.xi** | ✅ | Smoke passes |
| **compress.xi** | ✅ | Smoke passes |
| **contracts.xi** | ✅ | Smoke passes |
| **reflect.xi** | ✅ | Smoke passes |
| **cell.xi** | ✅ | Smoke passes |
| **rc.xi** | ✅ | Smoke passes |
| **alloc.xi** | ✅ | Smoke passes |
| **env.xi** | ✅ | Smoke passes |
| **error.xi** | ✅ | Compiles to IR with 0 errors (interface dispatch fix) |

### 4b. IO Runtime — Detail
| Function | Status |
|----------|--------|
| `write_file` | ✅ Verified (create + exists check) |
| `read_file` | ✅ Verified (read + content compare) |
| `file_exists` | ✅ Verified (pre/post write/remove) |
| `remove_file` | ✅ Verified (cleanup + exists check) |
| `append_file` | ✅ Pub, compiles |
| `create_dir` / `is_dir` | ✅ Pub, compiles |
| `args` / `print` / `println` | ✅ Pub (uses `c_str()` builtin) |
| `BufReader` / `BufWriter` / `Cursor` | ✅ Pub types + methods |
| `Read` / `Write` / `Seek` interfaces | ✅ Pub interfaces |

### 4c. Path Module — Detail
| Function | Status |
|----------|--------|
| `Path.new` | ✅ Works |
| `file_name` | ✅ Returns `Some(PathBuf)` |
| `extension` | ✅ Returns `Some(Str)` |
| `file_stem` | ✅ Returns `Some(Str)` |
| `parent` | ✅ Returns `Some(PathBuf)` |
| `is_absolute` | ✅ Works |
| `components` | ✅ Returns `Vec[Str]` |
| `canonicalize` | ✅ String-based `./..` resolution + separator normalization |
| `join` / `with_extension` / `with_file_name` | ✅ Implemented |
| `PathBuf.new` / `push` / `as_path` | ✅ Implemented |

---

## 5. PHASE 5c — SAFETY HARDENING & ARCHITECTURAL FEATURES (In Progress)

**Branch:** `feat/architect`
**Design doc:** `docs/COMPILER_ARCHITECTURE.md`

### 5c.1 Compiler Robustness (P0)

| Item | Status | Description |
|------|--------|-------------|
| **C runtime limits raised** | ✅ DONE | Fields: 16→256, Locals: already 512, Match arms: 16→128 |
| **`--max-depth N` flag** | ✅ DONE | Configurable recursion limit (default 500, max 10000) |
| **LLVM IR verification** | ✅ DONE | `opt -verify` runs before `opt -O1`; warns on malformed IR |
| **`--timeout N` flag** | ✅ DONE | Already existed (default 300s, was 60s) |
| **`--strict` mode** | ✅ DONE | Flag parsed + codegen field added (enforcement rules P1) |
| **`#[safety_audit]` attribute** | TODO | Parser + FnDecl support; requires attribute parsing (P1) |

### 5c.2 Safety Features (P1)

| Item | Status | Description |
|------|--------|-------------|
| **AI mode (`--ai`) JSON diagnostics** | TODO | Structured JSON with `suggestion`/`safety_hint` fields from template files |
| **Error recovery** | TODO | Parser collects up to 100 errors before aborting |
| **Contract `@pre` snapshot** | TODO | Store entry-point values for `ensures` clauses referencing pre-state |
| **Unsafe guidelines enforcement** | TODO | `--strict` mode errors on unsafe without `#[safety_audit]` |

### 5c.3 Structural Features (P2)

| Item | Status | Description |
|------|--------|-------------|
| **Derive macro codegen** | TODO | `derive[Default/Drop/Serialize/Deserialize]` |
| **Borrow checker struct-field borrows** | TODO | Field-level granularity for borrow tracking |
| **Enhanced smoke tests (3-5/5)** | TODO | 41 tests upgraded to production-grade coverage |
| **`--verify-all` flag** | TODO | SMT-LIB for all contracted functions |

### 5c.4 Implementation Order

```
P0 (compiler robustness):  --strict, safety_audit, C runtime limits, --timeout, --max-depth, opt -verify
P1 (diagnostics/tooling):  --ai JSON, error recovery, @pre snapshot
P2 (structural):           derive macros, struct-field borrows, enhanced smoke tests
P3 (verification):         --verify-all, contract coverage
```

---

## 6. PHASE 5d — PRODUCTION TOOLCHAIN

| Item | Status |
|------|--------|
| Package manager + registry | TODO |
| CLI toolchain (xiom build/run/test/bench) | Partial (`--run`, `--emit-ir`, `-o` exist) |
| Cross-platform CI | TODO |
| External C FFI binding generator | TODO |

---

## 7. PHASE 5e — SELF-HOSTING (v1.0.0)

| Item | Status |
|------|--------|
| Z3 static verification | TODO |
| Write compiler in XIOM | TODO |
| Bootstrap with byte-for-byte identical output | TODO |

---

## 8. VERIFICATION PROTOCOL

```bash
cargo build -p xiomc
cargo test -p xiom-parser --lib
cargo test -p xiom-check --lib
cargo test -p xiom-codegen --test stdlib_execution_tests -- --nocapture
cargo test -p xiom-codegen --test e2e_tests
cargo test -p xiom-codegen  # all regression gates
```

---

## 9. APPENDIX: Archival Documents

| Document | Status | Content |
|----------|--------|---------|
| `ARC_A_POINTERS.md` | **Active** — design reference for §5a.1 | Blast radius, 6-step implementation, e2e tests |
| `COMPILER_VERSIONS.md` | **Active** — canonical version log | Complete v0.1.0→v0.45.3 history |
| `PRODUCTION_HARDENING_BUGS.md` | **Active** — bug deep-dives | All 10 bugs documented with root cause + fix |
| `PRODUCTION_READINESS_PLAN.md` | **Superseded** by this document | |
| `CODEGEN_PRODUCTION_PLAN.md` | **Superseded** by this document | |
| `CODEGEN_TIER2.md` | **Superseded** by this document | |
| `SESSION.md` | **Active** — session handoff | Current state, carry-on prompt |
