<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Selfhost Plan -- Byte-Identical Self-Hosting (2026-08-11)

**Author:** Kilo (compiler-hardening session) | **Baseline:** fast suite 1112/1/1 (only the documented pre-existing `test_diff_test_produces_correct_ir` fails), e2e 2237/2240 -> **all 3 remaining failures fixed and verified** (m19_read_file_content, safety_probe, i2_parallel_codegen), stdlib-exec 72/72, warning gates 0/0.

---

## 1. Executive summary

The Rust-hosted compiler (`xiom.exe`) is production-grade: it compiles every
XIOM program in the repo, passes 2237/2240 e2e tests, and is warning-free.
**The XIOM-written compiler is NOT.** Today `selfhost/xiomc_v10.xi` is a
*function-signature extractor*: it parses fn headers and delegates body IR
emission to C runtime helpers (`xiom_fn_emit_all` etc.). `xiomc_v050.xi`
(809 KB) is an obsolete embedded-source experiment. The `full_diff_tests`
suite (Rust IR vs selfhost IR) is `#[ignore]`d and compares *feature counts*,
not bytes.

This plan takes the selfhost compiler from signature-extractor to a **100%
XIOM compiler that emits byte-identical IR to the Rust compiler**, compiles
itself (self1 == self2 by sha256), and passes the full e2e + full-diff suites.
It includes two interleaved optimization passes (O1: selfhost code quality,
O2: bootstrap-chain performance) -- both are required, not optional.

## 2. Honest current state (verified 2026-08-11)

| Component | State |
|-----------|-------|
| Rust compiler (crates/xiom-*) | Production-grade; 0 warnings; e2e 2237/2240 |
| selfhost/xiomc_v10.xi (9.5 KB) | Signature extractor + C-FFI body emission. NOT selfhosting |
| selfhost/xiomc_v050.xi (809 KB) | Obsolete embedded-source compiler -- archive/delete |
| selfhost/lexer_v2.xi, xiom-lexer.xi, xiom-parser.xi, xiom-check.xi, xiom-codegen.xi, xiomc.xi | Early stage modules (compile, but no parity gates) |
| full_diff_tests.rs | `#[ignore]`d; compares feature counts + fn names |
| Bootstrap e2e (v10 self-compile) | `XIOM_SELFHOST=1`-gated; v10 crashes on complex input |

### What "byte-identical" requires (the hard part)

Byte-identical IR means the selfhost emitter must reproduce the Rust
emitter's **determinism exactly**:

1. **Register numbering order** -- `%tmpN` increments in the same order
   (tmp_counter advances identically through every expression/statement).
2. **String interning order** -- `@.strN` numbering and `fctx.strings`
   emission order.
3. **Type-name generation order** -- `%struct.Tuple__...`, `%struct.__unsafe_ctx_N`,
   generic monomorphisation naming.
4. **Float literal formatting** -- `{:.17e}` (fixed in BUG 10; byte-stable).
5. **Module/catalog load order** -- the external-decl injection order.

Because XIOM's codegen will be a *port* of the Rust emitter's control flow
(not a reimplementation), determinism is achievable: the port preserves the
emit sequence. Any divergence is a parity bug, not a formatting choice.

## 3. Target architecture

The selfhost compiler mirrors the Rust crates 1:1, in XIOM:

```mermaid
flowchart LR
    S[source .xi] --> L[lexer.xi]
    L --> P[parser.xi]
    P --> C[checker.xi]
    C --> G[codegen.xi]
    G --> D[driver.xi + module catalog]
    D --> IR[LLVM IR text]
    IR --> RT[stdlib runtime C]
    RT --> EXE[native binary]
    subgraph runtime_ffi.xi
        S2[string ops: str_len/char_at/intern/concat]
        T2[fn table / symbol registry]
        F2[float formatting: {:.17e}]
    end
```

- `selfhost/src/*.xi` -- one file per stage, mirroring
  `crates/xiom-{lexer,parser,check,codegen}/src`.
- `selfhost/src/runtime_ffi.xi` -- pure-XIOM reimplementations of the C
  helpers v10 currently imports (`xiom_str_len`, `xiom_char_at`,
  `xiom_intern`, `xiom_fn_table_*`, `xiom_ir_*`). These must be ported to
  stdlib operations so the selfhost compiler has **no C dependency for
  codegen** (only the normal runtime for I/O).
- The **corpus** for all gates: `tests/regression/*.xi`, `examples/*.xi`
  (incl. `examples/diff_test.xi`, `examples/stdlib_smoke/*.xi`),
  `selfhost/_diff_phase1_*.xi`.

## 4. The parity gate (how byte-identical is proven)

`full_diff_tests.rs` is upgraded in Phase 0 to a three-tier gate:

| Tier | Compare | Use |
|------|---------|-----|
| T1 | feature counts + fn names | smoke gate during development |
| T2 | normalized IR (strip `%tmpN`/`@.strN` numbers) | intermediate gate |
| T3 | **exact line-by-line equality** (`rust_ir == self_ir`) | phase completion gate |

Plus the bootstrap gate: `xiom.exe selfhost/src/main.xi -o self1.exe`;
`self1.exe selfhost/src/main.xi -o self2.exe`; **sha256(self1) == sha256(self2)**.

## 5. Phases (each ends at a gate; checklists in docs/checklists/)

### Phase 0 -- Foundations (gate: T1 harness green on the corpus)
- Upgrade `full_diff_tests.rs` to T1/T2/T3 tiers; add the corpus manifest.
- Create `selfhost/src/` skeleton + `runtime_ffi.xi` (port the C helpers used
  by v10; verify behavior against the C versions via the existing v10 tests).
- Archive `xiomc_v050.xi` (move to `selfhost/archive/`).
- Checklist: `docs/checklists/selfhost-phase0.md`.

### Phase 1 -- Lexer parity (gate: token-dump equality for the corpus)
Port `crates/xiom-lexer` to `lexer.xi`. Add a `--dump-tokens` mode to both
compilers; gate on byte-equal dumps. TokenKind ordering and Span handling
must match exactly.

### Phase 2 -- Parser parity (gate: AST-dump equality)
Port `crates/xiom-parser` (incl. the 2026-08-11 generic-args heuristic fix
and M19 tuple/struct literals). Gate: `--dump-ast` byte-equal on the corpus.
This is the largest single phase; sub-slices: statements/exprs -> types ->
patterns -> modules/imports -> contracts -> generics.

### Phase 3 -- Checker parity (gate: diagnostics + type-annotation equality)
Port `crates/xiom-check` (T-gates, borrow checker, contracts, catalog/module
resolution). Gate: same diagnostics (order + text) and same accepted/rejected
set on the corpus.

### Phase 4 -- Codegen: function headers (gate: T3 header IR equality)
The current v10 scope, made byte-exact: fn signatures, param types, tuple
names, `alwaysinline`/`inlinehint` policy (size-based -- port `approx_block_cost`).

### **O1 -- Selfhost code-quality pass** (after Phase 4)
- Remove v10's "avoid the borrow checker" workarounds (`var done = 1 == 0`
  patterns) -- the checker parity from Phase 3 makes idiomatic code legal.
- Enforce contracts (`requires`/`ensures`) on the ported modules.
- Target: selfhost source compiles under `--strict` with zero warnings.

### Phase 5 -- Codegen: bodies, scalars + control flow (gate: T3 full-IR equality, scalar corpus)
Arithmetic, comparisons, if/while/for/match, calls, returns, strings, floats.
Port the Rust emitter's exact statement order.

### Phase 6 -- Codegen: structs, tuples, generics, unsafe (gate: T3 full-IR equality, whole corpus)
Struct/field lowering, tuple naming (BUG 1 fix parity), Option/Result
payloads, generics monomorphisation, the unsafe-block trampoline lowering
(captures, ctx structs, fault/retry paths -- port the 2026-08-11 fixes).

### Phase 7 -- Self-compile + bootstrap chain (gate: self1 == self2 sha256)
- `self1 = xiom.exe(selfhost/src/main.xi)`; `self2 = self1(selfhost/src/main.xi)`.
- Diff IR of self1 vs self2 on the corpus (T3).
- Un-gate `e2e_selfhost_v10_self_compile` (rename to v11+) and
  `e2e_selfhost_v10_self_compile_to_native`.

### **O2 -- Bootstrap-chain performance pass** (after Phase 7)
- Profile self1 compiling `selfhost/src/main.xi` (target: < 60 s on this
  machine; the Rust compiler does the corpus in ~10 s).
- Hot paths: string ops (avoid repeated concat), hash maps, fn-table lookups.
- Target: self1 compile-time within 2x of xiom.exe on the corpus.

### Phase 8 -- Full green (gate: full suite, nothing ignored)
- Un-ignore `full_diff_tests` (T3), run the full e2e suite on BOTH compilers
  (Rust-compiled and self1-compiled binaries) -- identical results.
- Fast suite + e2e + stdlib suites green for the selfhost binary.

## 6. Risks & mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Register-number determinism | Byte-identical IR fails | Port emitter control flow 1:1; T2 tier catches drift early; corpus grows per phase |
| String interning order | `@.strN` mismatch | Port `fctx.strings` collection order exactly; T3 gate per phase |
| C-FFI helper porting (formatting, fn table) | v10 semantics differ | Phase 0 runtime_ffi.xi with behavior tests vs the C versions |
| Recursion counter (500-depth trap) interacts with selfhost deep recursion | Bootstrap crashes | O1/O2 profile; raise/verify depth budget; keep the same counter semantics |
| Parallel codegen (I2) parity | Optional | Selfhost targets the SEQUENTIAL path first; I2 parity is post-Phase-8 |
| The documented diff-test failure | One red remains | It is a Rust-side test artifact (handoff: IGNORE); not a selfhost blocker |

## 7. Definition of done (all must hold)

1. `self1.exe` and `self2.exe` byte-identical (sha256) -- the compiler
   reproduces itself.
2. T3 byte-identical IR on the full corpus (full_diff_tests un-ignored,
   green).
3. The selfhost binary passes the full e2e suite (2237/2240 + documented
   ignore) with the same results as the Rust binary.
4. Zero C-codegen dependency (runtime_ffi.xi in pure XIOM).
5. Zero warnings on the selfhost sources under `--strict`.
6. Selfhost compile-time within 2x of the Rust compiler on the corpus (O2).
