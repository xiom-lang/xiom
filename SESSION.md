<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Handoff -- 2026-09-16 (compiler lane; rounds 61-83 in docs/SESSION.md)

2026-09-17 update (post-split, `main`): the registry-client findings
R32-R38 are FIXED -- terminal integrity gates on install, canonical
registry URLs + trust-store normalization, surfaced registry error bodies,
yanked-package handling, XIOM_HOME-aware package cache. Details and the
verification record are in docs/COMPILER_BUGS.md ("Registry-client
integration findings"). The registry repo's e2e harness gained the matching
locks (cache path + tamper/error-code/all-yanked assertions) -- test-only,
owned by the registry lane. Remaining supply-chain tail: transitive
dependency closure from registry metadata.

Also 2026-09-17: R39 FIXED -- same-leaf TYPE collision across project
modules (catalog injection now module-qualifies colliding non-generic type
leaves and rewrites all references; `crates/xiom-check/src/type_qualify.rs`,
lock `e2e_m84_type_same_leaf_modules`, CI lock line updated). The bench graph
no longer emits a bare `%struct.Metrics`/`%struct.Record`; its IR is
deterministic at 5,764,620 bytes.

Also 2026-09-17: R40 FIXED -- `derive[Clone]` on a pointer receiver
(`m: &M` -> `m.clone()`) passed the pointer where the by-value `%self` was
expected; LLVM accepted the mismatch silently and the clone was garbage. The
non-generic instance-method receiver coercion now loads the struct when the
callee's first param is a by-value struct (call.rs), lock
`e2e_m85_clone_ref_receiver` + CI lock line.

Also 2026-09-17: R41 FIXED -- generic-method calls passed struct VALUES where
a pointer self was expected (bench `Pair.read_first_Int_Int`); the generic
receiver ABI block now mirrors the non-generic materialization. R42 FIXED --
bare enum variants resolve scope-first (checker parity) so
`var c: Container[Int] = Empty` binds Container, not the first-declared
`Message`. See docs/COMPILER_BUGS.md R41/R42.

Also 2026-09-17 (stdlib-session sweep relay): R43 FIXED --
`&v` on a reference-typed local hard-errored C001, breaking the stdlib's
`var v = b; ... &v` pattern (x25519_keypair -> _bigint_to_le); `&*v == v`
now lowers to the stored pointer. Lock `e2e_m86_ref_of_reference`.
R44 RESOLVED STDLIB-SIDE -- `http_parse_response` invalid GEP was a stdlib
same-leaf collision; the stdlib renamed `net.net.HttpResponse` ->
`NetHttpResponse` (ce0c7fa) and verified probe + fuzz parser + batteries on
the pinned v0.60.0. R45 FIXED -- the single-param sweep's tuple mismatch
(tuple element names came from LLVM widths: Bool -> "Int", `as UInt16` ->
"Int16") plus the early-splice requirement (LLVM rejects alloca of a
forward-referenced named type). Lock `e2e_m87_tuple_element_types`; sweep
repro now compiles/links/runs. See docs/COMPILER_BUGS.md R43-R45.

Also 2026-09-17: STAGE 5 SUPPLY-CHAIN TAIL CLOSED -- transitive dependency
closure for registry installs (version-range matcher; cycle-safe closure
whose dependency source is the verified tarball's manifest; every artifact
verified) and transitive lockfile pinning (`Lockfile::from_resolved`).
Registry e2e 20/20 with a real dependency fixture. Driver hygiene landed:
temp names randomized per invocation/session; the stage-5 plan entries for
sandbox false-green (AUDIT #11), library `process::exit` (AUDIT #12), dbg MI
quoting (AUDIT #20) and fmt `defer` (AUDIT #10) were already closed and are
now recorded as DONE. Stage 5 remaining (feature-scale): clap-based arg
parsing, LSP incremental reparsing + cross-file index, fmt body-inline
comment trivia, cargo-vet audits.

Branch `main` (post-split). The round-83 slice (pre-split housekeeping +
handoff) and earlier rounds live in the pre-split history; the R32-R42
hardening slices are the latest compiler-lane commits.
Working tree should be clean; `.xiom_ai.json` is generated tooling state and
is now untracked/ignored (it has been committed before -- `git rm --cached`
in this round). In the monorepo phase the parallel stdlib lane committed to
the same branch and sometimes swept the whole tree (re-check `git log --stat`
if a change seems missing); after the split the stdlib is a separate repo and
its checkout at `stdlib/` is gitignored.

## State at handoff

Stage 3 Item A CLOSED (`strict_catalog_findings = true`), the R-bug queue
through R24 is CLEARED, and the supply chain is signed. Highlights:

- **R21 (flip closure, round 61)**: scope-first user aliases (pre-load
  worklist + single-segment `use X as Y;`), container-receiver wildcard
  guard, generic-`!` defer, `examples/test_mod` fixture collision. Locks
  `m74`; corpus gate live (checker 189/189).
- **R20 (round 62)**: catalog same-leaf alias delegation binds the
  checker-recorded owner-qualified target; deterministic suffix scans.
  Lock `m75_alias_delegation`.
- **R18 (round 63)**: contract payload reads on the bare `is` rebind; Err
  rebinds load Result field 2. Lock `m76`.
- **R16 (round 64)**: extern return types feed pointer inference
  (`buf + len` is pointer arithmetic). Lock `m77`.
- **R15b (round 65)**: same-leaf USER-program modules (module-qualified
  registration, collision-qualified symbols, full-path-first aliases).
  Lock `m78` (multi-source).
- **Catalog collisions (round 66, R21d)**: deterministic winner
  (source-dir index -> structural path match -> smallest canonical path),
  canonical identity, `warning[W001]` for loaded modules, `release/` skipped.
- **LSP (round 67)**: `Backend::parse_cached` (content-hash) + cross-file
  definition; lsp 44/44.
- **R22 (round 68)**: plain module imports bind for codegen receivers via
  `module_receiver_paths` (catalog-isolated); percent dedup unblocked.
- **Fuzz + sanitizer CI (round 69)**: standalone `fuzz/` cargo-fuzz
  workspace (lexer/parser/ctfe/pipeline + seeds); CI `fuzz-smoke` (ASAN,
  30s/target) + `sanitizer-smoke`; CI runs bin-only crate tests
  (pkg/dbg/lsp/mcp), robustness/fuzz suites, m74-m78 locks.
- **dbg async MI reader (round 70)**: reader thread + bounded-wait queues;
  non-stopping continues report "running" instead of hanging the DAP.
- **`.xi` DWARF (round 71)**: `DISubroutineType` (the old `type: !{}`
  dropped ALL DWARF) + per-statement `!DILocation`s under `-g`;
  llvm-symbolizer maps body lines. Lock `m79`.
- **R23 (round 73)**: fn-typed values are env-first in every shape
  (call-through-value, `Vec[fn()].pop()` payloads, struct fields). All async
  probes green; lock `m80`.
- **R24 (round 74)**: catalog-body externs win under isolation (program
  bodyless decls no longer shadow stdlib extern blocks); `extern_fns` is
  part of the per-body context. The Stage 7 selfhost compile gate
  (`diff_tests::test_selfhost_v092_compiles`) is GREEN.
- **Supply chain (round 75)**: ed25519 `signing.rs` (keygen/sign/verify +
  `~/.xiom/trusted_keys.json` trust store), fail-closed install for trusted
  registries, signed publish over ureq-only multipart (curl gone),
  `XIOM_REGISTRY_TOKEN` auth header, git deps must pin a full 40/64-hex
  commit. CLI: `xiom pkg keygen|trust|trusted|sign|verify`.
- **R25 CLOSED (round 77)**: the 30-module bench graph emits byte-identical
  IR (5,686,880 bytes, 5/5 runs; round-76 drifted ~130-230 bytes). Six
  HashMap-order classes fixed: `resolve_bare_fn_ref_key` (scope-first +
  deterministic suffix pick) wired into the fn-as-value ptrtoint,
  `wrap_fn_ref_env` and all five fn-typed-arg param lookups; fn-value
  ptrtoints map through `fn_symbol_map`; @pre snapshot worklist, mono
  worklist (total order), concrete-Option builtins and variant scans made
  deterministic. Lock `e2e_m81_fn_ref_same_leaf`; the determinism canary now
  covers BOTH selfhost v092 and the bench graph.
- **R27 CLOSED (round 78)**: installed-binary stdlib discovery. Pure
  `stdlib_candidates`/`is_stdlib_root` wired into the M12 bootstrap,
  `find_stdlib_dirs()` and `find_runtime_c()`; exe-relative `lib/` outranks a
  stale global `XIOM_HOME` (which is a fallback, not an override); catalog
  search dirs use the FIRST valid root only. Fake-install + release-binary
  `use xiom.io` sims exit 0; 5 new unit tests. Release R0 compiler-side
  blockers are done.
- **R31 CLOSED (round 82)**: cross-repo test isolation for the split. One
  helper (`xiom-graph::paths`): R27 candidates + `stdlib_root()`,
  `stdlib_smoke_dir()` (XIOM_STDLIB_SMOKES -> `<stdlib>/tests/smoke/` ->
  legacy), `stdlib_or_skip()`/`skip_if_missing()` (loud SKIP locally, hard
  FAIL under `XIOM_REQUIRE_STDLIB=1`). Driver delegates; JIT resolves runtime
  via the scan (`xiom build-runtime` verified); codegen/LSP/MCP tests
  parameterized; R9-01 guard prints SKIP; `scripts/fetch-stdlib.ps1|.sh` +
  `STDLIB_VERSION` (`main` for now -- release lane swaps in the split tag) +
  `.gitignore stdlib/`; packaging bundles from the checkout and drops the
  `xiom-playground` WASM copy for a release-artifact lookup.
- **Stage 6 start (round 76)**: `perf_budget_tests.rs` (IR byte budgets +
  180s ceiling + byte-identical determinism canary), CI-wired;
  deterministic variant->parent-enum and `type_meta` selection
  (`pick_deterministic`: current module -> shortest key -> lexicographic).

Last full gates (round 82): workspace `--lib` 521/521, checker 189/189,
feature-reg 510/510, stdlib-exec 85/85 (+2 ign), perf 2/2, pkg 52/52, dbg
34/34, lsp 44/44, mcp 39/39, jit 5/5, `xiom build-runtime` exit 0, e2e
**2331/2331** (R30 binary; the R31 run is the split gate), `cargo build
--release -p xiom` clean. Known red, stdlib-lane owned, pre-existing:
`stdlib_api_freeze_no_removals` (52 drifted signatures) and
`stdlib_tests::stdlib_all_modules_compile_to_ir` (`encoding.ascii85` T001) --
both documented in docs/COMPILER_BUGS.md R31 FIXED.

## R25 -- CLOSED (round 77)

Deterministic fn-REFERENCE resolution landed; the bench-graph drift is dead.
Full evidence and the fix breakdown are in docs/SESSION.md round 77. In
short: scope-first + `pick_deterministic` bare-fn resolver (lib.rs) used by
the fn-as-value ptrtoint / thunk symbol / fn-typed-arg param paths; the
ptrtoint now materializes the pre-assigned `fn_symbol_map` symbol; @pre
snapshot order, mono worklist total order, concrete-Option builtin order and
the variant-parent scans are sorted/deterministic.

Verify with:

```
target\debug\xiom.exe --emit-ir examples\benchmark\main.xi > a.txt   (x3, compare bytes)
cargo test -p xiom-codegen --test perf_budget_tests
```

Residual findings: the bare-variant disagreement was fixed by R42
(scope-first parent pick); the `%struct.Metrics` GEP by R39. The bench's next
clang error is the generic-arg inference leak -- see "Open compiler findings"
below. Stage 6 measures emitted IR bytes only, so it never gated; remeasure
before a full bench build.

Formal R25 entry in docs/COMPILER_BUGS.md is deferred (that file had
uncommitted stdlib-lane WIP at close; append once clean).

## Open compiler findings (pre-selfhost, not R0-blocking)

1. **Bench graph clang: generic-arg inference leaks a fixed-ARRAY type into a
   mono name** (next error after R41/R42). Repro:
   `var circles = [c.clone(), Circle{ radius: 3.0 }];` then
   `total_area(&circles, circle_area_fn)` in
   `benchmark.interfaces.test_interface_dispatch` emits the malformed symbol
   `@benchmark.interfaces.total_area_2 x Int(...)` (clang "expected '(' in
   call"). Two defects in one: the inferred `T` is the fixed-array type
   instead of the Vec's ELEMENT type (array literal -> M33 Vec conversion),
   and `monomorphised_fn_name` (lib.rs) does not sanitize concrete types, so
   the name is not a valid LLVM identifier. Fix shape: infer `T` from the Vec
   element for `&vec` callers bound from array literals; run concrete-type
   parts through `sanitize_container_arg` in every mono name. Bench is
   IR-gated, so no suite regresses today.

1b. **R44 same-leaf class -- resolved for HttpResponse, class remains**:
   the stdlib renamed `net.net.HttpResponse` -> `NetHttpResponse` (their
   `ce0c7fa`), so the http probe/fuzz parser are green. The other same-leaf
   groups (40 total; facade duplicates and genuinely conflicting
   collect/math/etc. types) still rely on first-wins; a dedicated
   compiler+stdlib slice (stdlib dedup first, then stdlib-wide R39
   qualification + their smoke battery) is the remaining work. Experiment
   evidence in docs/COMPILER_BUGS.md R44.

FIXED (R45): tuple element naming (Bool/`as` targets) + tuple defs spliced
into the type-decl block; lock `e2e_m87_tuple_element_types`; the stdlib
single-param sweep repro compiles/links/runs. Sweep-side exclusions for the
compiler run (stdlib/harness, not codegen): `xiom.os.env_unset` links
`unsetenv` (absent on Windows MSVC); `async_read_line(0)` passes a NULL
FILE* to `fread`; the run stops on an expected `requires` trip with dummy
args. See docs/COMPILER_BUGS.md R45.

FIXED (R43): `&v` on a reference-typed local now lowers to the stored
pointer (`&*v == v`) instead of C001 -- unblocks x25519_keypair /
_bigint_to_le; lock `e2e_m86_ref_of_reference`.

FIXED (R41/R42): generic pointer-self receiver ABI + scope-first bare-variant
resolution -- see docs/COMPILER_BUGS.md.

FIXED (R40): `derive[Clone]` on a pointer receiver -- the non-generic
instance-method receiver coercion now mirrors the by-value `%self` ABI
(loads the struct); lock `e2e_m85_clone_ref_receiver`.

FIXED (R39): the same-leaf TYPE collision -- catalog type qualification,
lock m84. See docs/COMPILER_BUGS.md R39-R43.

Fixed this session: R28 (temporary `.value`, lock m82), R29 (Vec in match arm,
lock m83), R30 (bare variant pick parity -- the old R25 residual; bench IR
byte-identical at 5,687,052 bytes pre-R39, 5,764,620 after).

## Working after the split (R31 contract)

The compiler repo and the stdlib repo are separate; the compiler repo no
longer contains stdlib sources:

- Fetch the pinned checkout: CI checks out `xiom-lang/stdlib` at the ref in
  `STDLIB_VERSION` (`.github/workflows/ci.yml` `Checkout stdlib at the pin`,
  release.yml likewise). The `scripts/fetch-stdlib.ps1|.sh` helpers referenced
  by the round-83 handoff are NOT in this checkout -- release-lane follow-up.
  Locally, clone into `stdlib/` or point `XIOM_STDLIB` at a checkout.
- Every cross-repo path resolves through the ONE helper
  `xiom_graph::paths`: `stdlib_root()` (XIOM_STDLIB -> exe-relative -> CWD ->
  XIOM_HOME fallback -> baked checkout), `stdlib_smoke_dir()`
  (XIOM_STDLIB_SMOKES -> `<stdlib>/tests/smoke/` -> legacy
  `examples/stdlib_smoke/`), and `stdlib_or_skip()` / `skip_if_missing()`.
- Missing checkout: loud SKIP lines locally; `XIOM_REQUIRE_STDLIB=1` (CI
  sets it) turns every skip into a hard FAIL.
- The stdlib lane owns moving the smoke corpus into `<stdlib>/tests/smoke/`;
  until it lands the legacy path keeps working.
- `stdlib/.gitignore` was added (generated runtime/smoke artifacts); it
  travels with the stdlib repo -- the stdlib lane may fold it into their
  bootstrap.
- Housekeeping done pre-split (round 83): ~13 GB of generated test binaries
  removed (repo root, `.test_build/`, `.testlogs/`, `tmp/`); `.gitignore`
  rebuilt (its tail had a corrupted UTF-16 block, so `.xiom_ai.json`,
  `.xiom_ai_cache/` and `xiom_verify_output.smt2` were never actually
  ignored); `.xiom_ai.json` untracked; harness temp sources (`_e2e_*.xi`,
  `e2e_*.xi`) ignored.

## Remaining queue (compiler lane)

1. **Supply-chain tail -- CLOSED (2026-09-17)**: transitive dependency
   closure landed. `select_version` matches `>=,<,<=,>,=,^,~` specs over
   non-yanked versions (exact pins still resolve yanked releases); install
   walks a cycle-safe deterministic `install_closure` whose dependency
   source is the VERIFIED tarball's own `package.xi` (index metadata as
   fallback), and every transitive artifact goes through the full
   verification path (sha256 + signature + lockfile). `lock` pins the
   transitive closure with digests (`Lockfile::from_resolved`). Server-side
   publish authentication is registry-side; `publish_package` is
   implemented. Registry e2e **20/20** (new core fixture + closure install +
   transitive lock assertions).
2. **clap migration** of the driver parser (large; keep the CLI surface
   byte-compatible, gate with the full e2e suite).
3. **fmt**: body-inline comment trivia attachment (stage-2 trivia dependency;
   shebang/header/string escaping already done round 42).
4. **LSP**: finish the cross-file index work behind the incremental AST
   cache (lsp 44/44 currently).
5. **Driver hygiene**: randomized temp names (the jit link dir is pid-based).
6. **cargo-vet audits** (cargo-deny already runs in CI; cargo-fuzz and
   ASAN/sanitizer CI landed round 69).
7. **Same-leaf TYPE collision -> FIXED (R39, 2026-09-17)**: catalog injection
   module-qualifies colliding non-generic type leaves and rewrites references
   (`type_qualify.rs`; lock m84). The next bench clang error is the generic-mono
   Pair ABI mismatch -- see "Open compiler findings".
8. **Stage 6 continuation**: real incremental engine, parallel
   monomorphization profiles, linker strategy, more budget metrics.
9. **Stage 7 selfhost ladder**: v092..v11 are milestone emitters, not yet a
   full XIOM-in-XIOM compiler; zero-ICE self-build is a multi-phase project.
   The selfhost COMPILE gate is green.

Cross-lane pending (stdlib lane, pre-existing): `stdlib_api_freeze_no_removals`
RED (52 drifted signatures since the 2026-08-07 snapshot) and
`stdlib_tests::stdlib_all_modules_compile_to_ir` RED
(`xiom.encoding.ascii85` T001). `STDLIB_VERSION` is now `stdlib-v0.60.0`
(release lane swapped it in).

## Workflow rules

- e2e/stdlib harnesses spawn `target/debug/xiom.exe` (release fallback):
  `cargo build -p xiom` after ANY checker/codegen change or the suites test
  a stale driver.
- Full e2e is ~18 min; run it for any resolution/codegen change. Fast gates:
  `cargo test -p xiom-check`, `--test feature_regression_tests`,
  `--test stdlib_execution_tests`, `--test perf_budget_tests`.
- Shared target dir can go stale (`cargo check` disagreeing with an isolated
  build): `cargo clean -p xiom-codegen -p xiom`, then rebuild the driver.
- PowerShell: capture `$LASTEXITCODE` immediately after each native call;
  `1> file` writes UTF-16 (use `[System.IO.File]::WriteAllLines`/`-Raw` when
  byte-exact IR/output is needed).
- ASAN-compiled programs need the LLVM runtime dir on PATH
  (`C:\Program Files\LLVM\lib\clang\22\lib\windows`); cargo-fuzz targets run
  locally only with a version-matched ASAN DLL -- use CI/Linux for those.
- Corpus triage: `cargo test -p xiom-check catalog_corpus_is_clean`;
  `$env:XIOM_CATALOG_DUMP='1'` prints every site.
- Stdlib boundary: monorepo phase -- never stage `stdlib/**` /
  `examples/stdlib_smoke/**` (parallel lane's files); post-split -- `stdlib/`
  is a gitignored checkout, never commit it. Report stdlib findings in
  `docs/COMPILER_BUGS.md` and in chat.

## Paste-ready prompt for the next compiler session

```
Continue the AXIOM compiler-lane readiness campaign in E:\xiom-lang\xiom
(the compiler repo; post-split, branch `main`). Read SESSION.md (repo root)
and docs/SESSION.md (rounds 61-83; round 83 is the pre-split housekeeping/R31
handoff) before touching code. The stdlib lives in its own repo: clone
`xiom-lang/stdlib` at `STDLIB_VERSION` into `stdlib/` or set `XIOM_STDLIB`
before any stdlib/smoke test; everything resolves through `xiom_graph::paths`
(XIOM_STDLIB, XIOM_STDLIB_SMOKES, XIOM_REQUIRE_STDLIB=1 in CI; CI checks out
the pin itself). Never commit the `stdlib/` checkout.

State: Stage 3 Item A CLOSED (strict catalog findings, checker 194/194),
R-bugs through R45 CLEARED (R32-R38 = registry-client findings; R39 =
same-leaf TYPE collision, lock m84; R40 = derive[Clone] on pointer receivers,
lock m85; R41 = generic pointer-self receiver ABI; R42 = scope-first bare
variants; R43 = `&ref` locals, lock m86; R45 = tuple element types +
early-spliced tuple defs, lock m87; R44 resolved stdlib-side; fixed on main
2026-09-17 with unit + e2e locks); e2e 2335/2335; supply chain signed AND
COMPLETE for the pre-registry phase: ed25519 keygen/trust/sign/verify,
fail-closed installs, ureq-only publish, git commit pins, TRANSITIVE
DEPENDENCY CLOSURE (range matcher + cycle-safe closure from the verified
manifest; `lock` pins the closure with digests; registry e2e 20/20); Stage 6
perf budgets wired (determinism canary covers selfhost v092 + bench graph;
bench IR 5,764,620 bytes), selfhost v092 compile gate GREEN, release R0
compiler-side blockers DONE (R25+R27+R31+R39-R45, release build clean). The
bench graph's generic-arg inference leak (array type in a mono name) is the
only OPEN compiler finding -- see "Open compiler findings".

Pending (cross-lane): stdlib_api_freeze_no_removals RED (52 drifted
signatures) and stdlib_tests::stdlib_all_modules_compile_to_ir RED
(xiom.encoding.ascii85 T001) -- stdlib-lane owned, documented in
COMPILER_BUGS R31 FIXED. STDLIB_VERSION is `stdlib-v0.60.0`.

Your task, in order (Stage 5 completion; the supply-chain tail and driver
hygiene are CLOSED; the stdlib sweep findings R43/R45 are FIXED and R44 is
resolved stdlib-side):
1. Fix the last open compiler finding: generic-arg inference leaks a
   fixed-array type into the mono name (`total_area_2 x Int`), then make the
   bench graph clang-clean (repro in "Open compiler findings").
2. R44 remaining class (coordination slice): the stdlib dedups the genuinely
   conflicting same-leaf declarations, then land stdlib-wide qualification
   in R39's pass with their smoke battery green. Until then, stdlib leaf
   collisions keep first-wins.
3. clap-based arg parsing (keep the CLI surface byte-compatible); LSP
   incremental reparsing + cross-file index; fmt body-inline comment trivia;
   cargo-vet audits.
4. Stage 6 continuation (incremental engine, parallel mono profiles, linker
   strategy) and the Stage 7 selfhost ladder -- both on their own branch
   after the public release gates.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Capture $LASTEXITCODE
right after each native command. Use --emit-ir / --sanitize=address for
miscompile work. Keep commits atomic (code + docs together); never commit
stdlib/** or the stdlib/ checkout.
```
