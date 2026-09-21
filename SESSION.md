<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
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
R44 SLICE LANDED (2026-09-18) -- `http_parse_response` invalid GEP was a
stdlib same-leaf collision; the stdlib renamed `net.net.HttpResponse` ->
`NetHttpResponse` (ce0c7fa) and the compiler now includes catalog modules
in the collision qualification under the shape-conflict standard
(all-catalog groups only), so the collision cannot resurface: pin probe +
negative control, lock `e2e_m90_stdlib_same_leaf_http`, corpus 949/949.
R45 FIXED -- the single-param sweep's tuple mismatch
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

2026-09-18 update (post-split, `main`): R46 LANDED -- the bench graph is
**CLANG-CLEAN** (`xiom --emit-ir examples/benchmark/main.xi | clang -c -x ir
-` exits 0; deterministic at 5,808,645 bytes; locks m87-extended/m88/m89).
The stdlib lane verified R43/R44/R45 probes and check_modules 509/509 on
483f283e, and reported the full unmodified sweep as "1 fast clang failure +
3 hangs >5-15 min". Re-measured here: with the RELEASE driver the sweep's
codegen+clang finish in **481.6s** and fail only at link
(`undefined symbol: unsetenv`, the stdlib Windows gap); the debug driver
exceeds its 300s watchdog, which is what looked like a hang. No compiler
hang. Remaining OPEN compiler finding: none from the R46 family -- R46b
(2026-09-18) closed the cross-module qualified-receiver and receiver-only
generic method instantiation residuals; m88 now locks the direct call forms
(wrappers removed). **R44 qualification slice LANDED (2026-09-18)**: catalog
(`xiom.`) modules now participate in the R39/R46 collision triage under the
stdlib audit standard -- all-catalog groups qualify only when declared
shapes conflict (facade duplicates keep the legacy key); project-owned
groups keep the R39/R46 rule, so user emission is untouched (bench IR
byte-identical at 5,808,645). Verified: `p_http_resp_codegen` green on the
pin (negative control reproduces the clang GEP error), check_modules
509/509, smoke corpus 949/949 on stdlib main, stdlib_exec 85/85 on the pin,
lock `e2e_m90_stdlib_same_leaf_http`. Remaining stdlib-side work: their
16-group dedup worklist (hygiene now, not a correctness prerequisite).
Stdlib status
for the release lane: their pin is v0.60.0 and release tag v0.60.1 predates
R43/R45/R46, so `COMPILER_VERSION` moves only when a release contains them
(their nightly already tests main).

Branch `main` (post-split). The round-83 slice (pre-split housekeeping +
handoff) and earlier rounds live in the pre-split history; the R32-R46
hardening slices are the latest compiler-lane commits.
Working tree should be clean; `.xiom_ai.json` is generated tooling state and
is now untracked/ignored (it has been committed before -- `git rm --cached`
in this round). In the monorepo phase the parallel stdlib lane committed to
the same branch and sometimes swept the whole tree (re-check `git log --stat`
if a change seems missing); after the split the stdlib is a separate repo and
its checkout at `stdlib/` is gitignored.

## Current state (2026-09-19, post-R48) -- READ FIRST

Everything after this section is the pre-R31/r31-r83 history. Live state:

- **Stage 5 is COMPLETE**: clap owns the flag surface (steps 1+2), cargo-vet
  gate (175 exempted), fmt body-inline comments, LSP cross-file index +
  parse cache, driver temp hygiene, help parity (full `--help`, `--strict-mode`
  alias), CRB-1..CRB-5 done with the ops batch, AI-mode audit (installer
  config fixed, `--ai-local` enforced, HTTPS/plaintext key guard, structured
  `FIX:/WHY:/Confidence:` hint fields).
- **Compiler bugs cleared through R48** (docs/COMPILER_BUGS.md is the
  authority). Fixed in R48: interface dispatch through `&T`/`&mut T` generic
  args (9 playground L6 lessons), pointer/double match-result zero-init,
  script-cache build identity, native `xiom fmt|lsp|mcp|pkg|dbg|verify|ffigen`
  dispatch, WASM release asset. Lock `e2e_m92_interface_dispatch_zero_init`.
- **R49 batch LANDED (2026-09-19)**: playground R48-residue fixes --
  L6-28 (struct-literal generic inference + annotation type args + concrete
  Option registration + boxed struct-element load/unbox + clone type),
  L6-31 (duplicate definition dedupe), L8-15/L8-18 (computed Vec receivers,
  consistent box/unbox, inline set memcpy), L6-05 (mono param Vec element
  types), L2-19 (qualified enum variants), L0-11/array-of-Str (i8* element
  type), L5-32/L5-34 (match slots keep Str + Some/Ok payload unbox),
  L5-42 (bare conversion-call return type), and the stdlib-relayed
  `@pre` call-capture bug (all-ident collection + pointer-slot rebind for
  ref params + Vec-buffer deep snapshot). Locks `e2e_m93..m98` + CI lock
  line; fixtures `tests/regression/m9[3-8]_*`.
- **Verified on the R49 batch**: full e2e **2346/2346** (locks m93-m98
  included), checker 195/195, feature-reg 510/510, and the
  `stdlib_api_freeze_tests` gate GREEN after the R49-1 resolver fix +
  snapshot regen. The earlier `56b6e0e3` record (2340/2340, feature-reg
  510/510, checker 195/195, robustness 63/63, fuzz 24/24, perf/determinism
  2/2, fmt 86/86, lsp 45/45, stdlib-exec 85/85 (+2 ignored), xiom lib
  32/32, pkg/dbg/mcp 63/34/39, `cargo deny` + `cargo vet` clean, ASCII
  guard, bench IR 5,808,645 bytes + `clang -c` exit 0) still holds for the
  unchanged lanes; workspace version 0.61.0.
- **Release lane**: `release.yml` ships all nine CLI tools + pinned z3
  (tools/z3-pins.json, glibc floor 2.35) + `xiom-wasm-<ver>.wasm`; guard
  (tag == workspace version, ancestor of main) and the `compiler-release`
  docs dispatch (`XIOM_RELEASE_TOKEN` must cover xiom-lang/website) are
  wired. Pending: stdlib lane green -> bump `STDLIB_VERSION` in the release
  PR -> tag `v0.61.0`; optional rewrite of the protected tags
  `v0.60.0`/`v0.60.1` (old objects remain on the remote).
- **R54 / R49-4 CLOSED (2026-09-21)**: the clang 22.1.8 ISel crash is fixed
  in codegen -- large fixed-array locals (>= 16 KiB) are zero-initialized
  with `llvm.memset` instead of an aggregate `store zeroinitializer`, and
  indexed access uses their address (no whole-array value materialization).
  `p_sweep_single_param.xi` compiles AND links in ~56 s (previously clang
  ISel 0xC0000005); lock `e2e_m109_large_fixed_array`. Windows-pin note:
  the pinned stdlib's `os/env.xi` still calls `unsetenv` directly, so the
  probe links on Windows only after the pin moves to the
  `xiom_env_set/unset` shim revision (bump `STDLIB_VERSION`).
- **R53 registry metadata consumption (2026-09-21, registry relay)**:
  `xiom-pkg` now consumes the server-extracted package metadata
  (`license`, `categories`, `keywords`, `repository`) in the index and
  `/packages/:name`; `search` matches keywords/categories, prints them, and
  takes `--category <c>` (local filter) plus `--json`; new `info
  <pkg>[@version]` renders description/categories/keywords/license/repo,
  the pinned version's digest/signature and the sorted version list (human
  and JSON); publish prints 201-body `warnings`. Manifest parsing tolerates
  inline and multi-line metadata arrays (locked by test). MCP gained
  `search_packages({query?, category?})` and `package_info(name)` (16 tools
  total) invoking `xiom-pkg --json`, returning the exact index field names.
  Tests: pkg 67/67, mcp 39/39. Live smoke: search/info JSON + `xiom pkg`
  dispatch verified against the staging registry.
- **R52 payload/binding batch (2026-09-19, playground audit §19)**: closed
  the remaining 11 nondeterministic/pointer-print lessons (L3-02,
  L5-09/20/24/26/29/31/35/36/43) plus L5-21 via a family of type-erasure
  fixes: concrete generic-struct field/method typing, match-result and
  match-expression Str prediction, Map[Str,Str]/unannotated-Some payload
  binding, redundant explicit-self argument handling, and single evaluation
  of side-effecting match scrutinees. Locks `e2e_m106..m108`; e2e
  2356/2356. Still open: L3-50 (Result tuple payload path) and L8-14
  (Map[Str,Str] morse trap).
- **R52 verifications (2026-09-19, packages + website relays)**:
  `xiom-verify --check` DOES run the bundled/auto-detected Z3 over the
  generated SMT-LIB (Z3Runner: Z3_PATH -> `<exe_dir>/[../bin/]z3[.exe]` ->
  PATH; verdicts Proven/Violated/Inconclusive/Error), while `xiom --verify`
  (and `xiom-verify` without `--check`) is EXPORT-ONLY (writes SMT-LIB; only
  prints "Z3 found -- use 'z3 file.smt2'"). Website `compiler.md` line ~81
  ("`--verify` runs Z3 over that") is therefore inaccurate -- the corrected
  wording (exported vs checked vs proved) is relayed for the website lane.
  `cargo build -p xiom --features nasm` assembles the runtime .asm objects
  (`stdlib/runtime/*.obj` refreshed by nasm 3.02) and links/runs the asm
  memop path (m96 fixture exit 0), so both configurations now work. Release
  artifacts rebuilt from source: `target/release/xiom.exe pkg --help` execs
  the sibling xiom-pkg (was the stale pre-R48 binary) and `pkg keygen
  --help` prints the per-command usage; release compile smoke exit 0.
- **R52 packages relay (2026-09-19)**: (a) unqualified `assert` after
  `use xiom.test;` returned a corrupt TestResult (F/0) because the keep-first
  bare alias bound the transitively-imported private `core.assert`; bare-call
  resolution now only honours a keep-first alias whose target is PUB and
  otherwise ranks imported-module exports (private helpers inside their own
  module, e.g. `normalize_duration`, stay reachable). Lock
  `e2e_m105_unqualified_assert`. (b) `xiom.std` (and legacy `xiom-std`) is a
  platform dep: a version spec no longer fails the registry closure. (c)
  `xiom pkg <cmd> --help` prints that command's usage (`keygen --help` no
  longer writes a key). (d) The local `target/release/xiom.exe` is stale vs
  source (pre-R48 dispatch); a fresh release build is needed for publishing
  -- the source is correct.
- **R51 stdlib residual (2026-09-19, p_pre_capture_callee)**: implication-
  wrapped contract clauses (`result is Some => ...@pre...`) never emitted
  entry snapshots -- the `@pre` walkers had no `Imply`/`Is` arms, so the
  ensures compared the live pointer with itself (2 == 2 - 1 in list/queue/
  rbtree/fenwick). Both walkers now descend; `p_pre_capture_callee.xi` and
  `p_wave8_shapes.xi` exit 0; lock `e2e_m104_pre_capture_callee`. The stdlib
  lane can restore the strong `@pre` size clauses in the blocked modules.
- **R51 L4 cluster (2026-09-19, playground audit §19)**: closed the four
  remaining `L4-*` clang failures -- `@pre` receivers in `.len()`
  (`items@pre.len()`, `self@pre.items.len()`) now resolve through the
  Vec/field type helpers, and `for i in range(0,n)` resolves the range
  builtin structurally so a user `fn range(...)` cannot hijack the loop.
  The playground's C17 residue is now L6-40 (needs the fixpoint inference
  pre-pass) and L5-40 (container-ABI design, see the L5-40 entry). Locks
  `e2e_m101..m103`.
- **R51 playground requests (2026-09-19, audit §19)**: `--opt-level N` is now
  honored on the script-run path (`xiom run [--opt-level N] file.xi`) and is
  part of the script-cache key (`xiom-cache-v3|...|opt=N`), so an -O0 run is
  never served an -O2 binary; the flag is also stripped from the positional
  args (it used to be read as the file name). An empty/unset HOME or
  USERPROFILE now falls back to `%TEMP%/xiom_jit` (the cache kept working;
  earlier an empty HOME produced a cwd-relative path). Verified on the lesson
  corpus shape: opt=0 and opt=2 builds populate separate entries, cached
  reruns 0.03-0.08s, HOME-less cached rerun works. Tests: xiom lib 32/32,
  cli_args 1/1; jit cache-key test covers level separation.
- **R50 registry follow-ups (2026-09-19, relayed by the registry session)**:
  `xiom install` now delegates to the verified `xiom pkg install` client and
  `xiom update` is retired with guidance -- the legacy git-clone-from-
  `/packages.json` handlers (`handle_install`, `fetch_registry_index`,
  `parse_deps_from_manifest`, `RegistryPackage`) were deleted so no install
  path can skip checksum/signature/yank. Package names are dotted: the
  resolver accepts canonical `xiom.std` and the legacy `xiom-std` alias
  (the pinned stdlib checkout still declares the hyphen form; coordinate the
  full switch with the stdlib session). Tests: pkg 64/64, xiom lib 32/32,
  mcp 39/39, graph 31/31, cli_args 1/1. Queued: `xiom pkg update|outdated`
  reading /index.json and a SHA256-verified self-update from
  dl./latest.json; OIDC publish needs no client change.
- **OPEN compiler bugs after R49** (exact repros/evidence in
  docs/COMPILER_BUGS.md; reproduce from the playground lesson sources):
  1. L6-40: module-scoped `Runner[T: Plugin]` `create()` -- T is only fixed
     by a LATER call; needs a fixpoint pre-pass (single-pass inference
     emits the `0` fallback). Trace evidence captured.
  2. L5-40: builds now, but `group_by_age` prints 0 -- Map container-payload
     ABI (insert boxes the Vec then memcpys the unboxed header into an
     8-byte handle slot; get derefs the handle).
  3. C18/C19 residue: remaining per-lesson triage (L3-02,
     L5-09/20/24/26/29/31/35/36/43, L3-50 exit 200, L8-14 `0xC000001D`,
     L5-21 Float64 bits inside `Vec[T]`).
  4. R49-1 module-path alias: FIXED (declared-identity parsing + import
     rewrite + freeze-resolver header index). `stdlib_api_freeze_tests` is
     GREEN (214/214 frozen entries; 52 stale/renamed snapshot lines
     regenerated in the same commit). Lock `e2e_m99_module_path_alias`.
  5. R49-3 Result payload contract: FIXED (payload rebound `.value`
     container dispatch; lock `e2e_m100_result_payload_contract`).
     R49-4 (clang ISel crash) still filed.
  6. Perf: `.to_str()` auto-injected `xiom.fmt` closure +2.3-2.5 s ->
     reachable-function-only peek (Stage 6 gate).
  7. C3 (script-mode flags) / C6 (stdlib `package.xi`): need an exact
     playground repro before changing behavior.
- **Final-IR tip**: `--emit-ir` prints the INTERMEDIATE emitter output; to
  get the IR clang actually compiles, force the link to fail
  (`--link missing_xyz`) and read `<output>.ll` (kept on failure, deleted
  on success). A later pass rewrites e.g. `Task.to_str` stubs to
  `Str.to_str`.
- **Playground repro recipe**: `git clone --depth 1
  https://github.com/xiom-lang/playground tmp/playground`; each lesson
  `lessons/**/Lx-yy.json` carries `.solution`; extract to `tmp/lessons/` and
  run `target\debug\xiom.exe -o tmp\lessons\Lx-yy.exe tmp\lessons\Lx-yy.xi`
  (rebuild `cargo build -p xiom` first; stdlib resolves from `stdlib/`).
  Do NOT rebuild while an e2e run is active: the harness spawns
  `target/debug/xiom.exe` and a mid-run replacement produces flaky
  crashes/false failures.

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
(scope-first parent pick); the `%struct.Metrics` GEP by R39; the generic-arg
inference leak by R46. **The bench graph is now CLANG-CLEAN** (R46):
`xiom --emit-ir examples/benchmark/main.xi | clang -c -x ir -` exits 0, and
the determinism canary stays green at 5,808,645 bytes. Stage 6 measures
emitted IR bytes only, so this never gated; it is now also a full-build
candidate.

Formal R25 entry in docs/COMPILER_BUGS.md is deferred (that file had
uncommitted stdlib-lane WIP at close; append once clean).

## Open compiler findings (pre-selfhost, not R0-blocking)

1. **Generic method instantiation misses for qualified receivers** (R46
   residual) -- **FIXED 2026-09-18 (R46b); m88 wrappers REMOVED**. (a) A
   `module.Type` receiver resolved its same-leaf type by `type_meta` HashMap
   order, so `g.Box.new` bound the sibling module on ~25% of runs and fell
   to an erased zeroinitializer stub. The receiver path is now expanded
   through the checker's module bindings (`qualified_type_key_for_path`,
   deterministic). (b) A generic method whose type parameter comes only from
   the receiver (`value_of()`/`is_sealed()` on `Box[Int]`, and computed
   receivers like `g.Box.new[Str]("x").value_of()`) now infers the type arg
   from the receiver call's instantiation (`receiver_generic_arg_at`)
   instead of the literal-0 stub. Lock `e2e_m88_generic_same_leaf_boxes`
   covers the direct cross-module call forms. See docs/COMPILER_BUGS.md
   R46b.

1b. **R44 same-leaf class -- resolved for HttpResponse, class remains**:
   the stdlib renamed `net.net.HttpResponse` -> `NetHttpResponse` (their
   `ce0c7fa`), so the http probe/fuzz parser are green. The other same-leaf
   groups (40 total; facade duplicates and genuinely conflicting
   collect/math/etc. types) still rely on first-wins; a dedicated
   compiler+stdlib slice (stdlib dedup first, then stdlib-wide R39
   qualification + their smoke battery) is the remaining work. Experiment
   evidence in docs/COMPILER_BUGS.md R44.

FIXED (R46): **bench graph clang-clean** -- generic same-leaf collisions are
shape-triaged, bare literals that are both struct and enum variant bind by
field names, mono tuple params use the concrete substitution, mono names are
identifier-sanitized, and method-receiver variants resolve leaf-scope.
Locks m87 (extended), m88, m89; bench IR deterministic at 5,808,645 bytes.

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

## Cross-lane notes (2026-09-18, compiler-lane answers)

**Website `docs/language/compiler.md` review.** `xiom-lang/website@c3d74f24`
(main) does NOT yet contain the consolidated Architecture/Modes sections or
the three flags; no open PR carries them. Verified facts to land them:

- `--runtime-contracts` (main.rs:388, help:1139): forces runtime contract
  guards even in release (`check_contracts || runtime_contracts`,
  lib.rs:627/1039). Release strips contracts without it.
- `--keep-debug-checks` (main.rs:391): release strips debug intrinsics
  (`set_strip_debug_checks(release && !keep_debug_checks)`, lib.rs:631/1042);
  the flag retains them. It was MISSING from `xiom --help` -- fixed
  2026-09-18 (now listed).
- `--enable-unsafe-direct` (main.rs:420-426, help:1134): allows
  `#[unsafe_direct]` in USER code; stdlib/trusted packages may use it
  without the flag (codegen context.rs:109-116, decl.rs:1139). Prints a
  warning when set.
- Crate list: **20** crates (Cargo.toml members): xiom, xiom-ast,
  xiom-check, xiom-codegen, xiom-ctfe, xiom-dbg, xiom-display, xiom-doc,
  xiom-ffigen, xiom-fmt, xiom-graph, xiom-jit, xiom-lexer, xiom-lowering,
  xiom-lsp, xiom-mcp, xiom-parser, xiom-pkg, xiom-verify, xiom-wasm. The
  website's "Architecture Decisions" row says 19 and lists a non-existent
  `cli` crate while missing `xiom-lowering`; `xiom` IS the CLI.
- v0.57 row verified: `#[unsafe_no_retry]` / `#[unsafe_direct]`
  (codegen), guard-heap arena + Copy-Out + guard pages
  (emitter.rs:904-912), SEH `__try/__except` trampoline + once-only
  transient retry (expr.rs:4973/4999), zero-escape gates T002/T003/T005/
  T006/T007 (checker, 35 sites).
- v0.58 row verified: numeric policy "cannot mix Int with Float64 --
  convert explicitly with `as`" (checker lib.rs:5421+), labeled loops
  (`@label:` parser:1131), debug intrinsics `dbg!`/`todo!`/
  `unimplemented!` (parser:2329) + `assert`/`debugger`, release stripping +
  `--keep-debug-checks`, `else if` accepted (empirical: check+compile+run
  probe), sublib-prefix resolution (checker lib.rs:146).
- AI_CONTEXT.md metadata refresh (website owns the file): version
  **v0.61.0**; compiler gates e2e 2338/2338, checker 195/195,
  feature-reg 510, robustness 63, fuzz 24, perf/determinism 2/2, fmt 86,
  lsp 45; stdlib pin `stdlib-v0.60.0`: 517 `.xi` files under `xiom/`,
  509 check_modules probes, 6,532 `pub fn` lines (the "512 modules /
  6,379 pub fns" line is stale). v0.57/v0.58 "New in" rows stay accurate;
  add a v0.59-v0.61 row for the repo split, R39/R44/R46/R46b same-leaf
  qualification, supply chain closure, single-source version 0.61.0,
  cargo-vet gate, and `xiom-pkg` in the archives.

**Registry lane**: `xiom-pkg` IS shipped now. `release.yml` builds
`-p xiom -p xiom-pkg`, stages both binaries in the Windows/Linux/macOS
archives, and asserts the staged `--version` of each before archiving
(CRB-3); the installer wrapper also dispatches `xiom pkg`. Verified
locally: `release/xiom-v0.61.0/bin/xiom-pkg.exe --version` -> v0.61.0.

**Installer / z3 ownership (superseded by CRB-3b below)**: the compiler repo
owns the installer and the
bundle LAYOUT -- `package.ps1` / `package.sh` / `tools/installer/*` build
the portable folder and copy `z3.exe` into `bin/` when one exists at
`target\release\z3.exe` or `%TEMP%\z3.exe`; `xiom-verify::find_z3`
auto-detects a bundled z3, common install paths, or PATH. The compiler repo
does NOT download or vendor z3, and the CI release workflow stages no z3,
so ACQUISITION/provisioning (and whether official archives bundle it) is
`/ops` release-side. Point the question at ops for z3; installer scripts
stay here.

**Playground lane (2026-09-19 handoff, C18/C19) -- FIXED (R47)**: the
`.to_str()`/container Str garbage and Float64-bit-pattern outputs traced to
four codegen/checker defects (conversion methods never injected without
`use xiom.fmt`; `unwrap_or` phi dominance violation; `ptrtoint double`
invalid cast; erased i64 payload/ABI type loss on chained receivers). All
fixed on `main`; lock `e2e_m91_conversion_methods` covers the full matrix
without importing `xiom.fmt`. See docs/COMPILER_BUGS.md R47. Playground
after updating the toolchain: `node tools/generate-expected-outputs.js
--wsl` then `node tools/lesson-audit.js --baseline
tools/lesson-baseline.json`. C17 (31 lessons cannot run at all) is NOT
confirmed compiler-side -- the audit's suggested next step; ask them to
re-run it on the fixed toolchain before assigning.

**Installer / z3 ownership -- RESOLVED (CRB-3b, 2026-09-19)**: ops decided
every archive ships all nine CLI tools and the pinned z3 solver. The
compiler repo owns the pin (`tools/z3-pins.json`, z3-4.13.4 SHA256-verified
per platform, Linux glibc floor 2.35), the platform-aware
`Z3Runner::find_z3` (bundled sibling `z3[.exe]`, `Z3_PATH`, common installs,
PATH; `xiom doctor` prints the resolved path), and the release.yml
bundle/stage/assert steps. Windows archives carry the app-local VC runtime
DLLs z3.exe needs; macOS carries `libz3.dylib`; both carry `LICENSE-Z3`.
`package.ps1`/`package.sh` still copy a z3 found at `target/release` or
`%TEMP%`; the CI path is the pinned download.

**CRB-3b extras**: every tool now self-reports (`xiom-mcp`/`xiom-dbg`/
`xiom-lsp` gained `--version`/`--help` before their stdio loops), and
`[profile.release] strip = true` shrinks the nine release binaries. The
staged-layout assert loop runs `--version` on all nine plus `z3 --version`
before each archive is sealed; CRB-4 guard and CRB-4b dispatch unchanged.

**Website help-parity follow-ups -- DONE (2026-09-19)**: `--help-ai` now
states the real LLM timeout default (10 s, matching `--help` and the
`unwrap_or(10)` in main.rs); `xiom --help` gained a "MORE OPTIONS
(advanced)" block listing every real flag it omitted (--check,
--emit-tokens, --debug/-g, --release, --opt-level, --lto, --cache,
--no-cache, --jit, --lazy, --parallel-codegen, --strict, --strict-mode,
--strict-exhaustive, --static, --standalone, --max-depth, --count,
--bench-file, --test-dir, --test, --scaffold, --clean, --explain,
--registry, --locked, --frozen, --ai-silent, --batch) plus repl /
build-runtime subcommands and the launcher's tool dispatchers; `--strict-mode`
is now an accepted alias of `--strict` (docs spelling) with a unit test.

**CRB-3c -- XIOM_HOME alignment DONE (2026-09-19)**: `xiom_graph::paths`
now owns the installed-home resolver: `XIOM_HOME` wins when set, otherwise
the first EXISTING candidate from the canonical installer layout
(`%LOCALAPPDATA%\xiom` / `$XDG_DATA_HOME|~/.local/share/xiom`) then legacy
`~/.local/xiom`, `~/xiom`; when nothing exists it returns the canonical
default so diagnostics stop guessing `~/xiom`. `xiom doctor` and `xiom doc`
use it (`doctor` prints the searched list when the stdlib is missing), and
the root source installer now installs to `~/.local/share/xiom` like the
shipped installer. `xiom-graph` 31/31 (new candidate-order/dedup test);
doctor smoke: canonical install found with no env, XIOM_HOME override
honored.

**Ops release checklist state**: CRB-1 (single-source 0.61.0), CRB-2
(banner/literal sweep), CRB-4 (tag==workspace-version + on-main guard),
CRB-4b (compiler-release docs dispatch), CRB-5 (package scripts read the
workspace version, never rewrite Cargo.toml) are all DONE and on
`origin/main`; CRB-3 (xiom-pkg) and CRB-3b (nine tools + pinned z3) are
accepted. The ops list repeating 1/2/4/4b/5 predates those pushes.

**AI-mode audit (2026-09-19)**: the installer's AI configuration was DEAD
(it wrote `KEY=VALUE .xiom_ai_config` with `XIOM_AI_API_KEY`, while the
compiler only read JSON `.xiom_ai_config.json` from cwd/home), and
`--ai-local` was parsed but never enforced. Fixed:

- both installers write JSON to `$XIOM_DIR/.xiom_ai_config.json` (the
  compiler now also searches `XIOM_HOME`), chmod 600 on Unix plus a
  plaintext-key warning; the shell wrapper exports `XIOM_HOME`;
  `.xiom_ai_config*` is gitignored.
- `finalize_config` enforces the `--ai-local` contract: Ollama + loopback,
  cloud key dropped, cloud-default model swapped to codellama; a remote
  endpoint is refused.
- a non-empty API key is refused over plaintext http:// to non-loopback
  hosts (`XIOM_AI_ALLOW_HTTP=1` opt-out for trusted proxies); the
  non-silent run prints the endpoint and whether env or a config path
  supplied it.
- prompts now carry the diagnostic MESSAGE + file + compiler version, mark
  code snippets as untrusted input (prompt-injection hardening), and ask
  for `FIX: / WHY: / Confidence:`; `XIOM_AI_TIMEOUT` and
  `XIOM_AI_MAX_TOKENS` are wired and documented; docs/AI_PIPELINE corrected
  (the cache "random session salt" claim was false; install config and HTTP
  rule documented).
- legacy `xiom install` / `xiom update` / `xiom registry` now use
  `<XIOM_HOME>/packages` and `<XIOM_HOME>/registry.json` (an explicit
  XIOM_HOME always wins; an existing legacy `~/.xiom/registry.json` is
  still honored) so the old spelling sees `xiom pkg` installs.

**Playground R48 (2026-09-19) -- fixed 3 classes, cache hazard, fmt dispatch,
WASM asset**: interface dispatch through `&T` generic args was mono'ing as
`Int` (type_from_ast strips the ref; the bare-T branch had no Ref arm) and
now resolves the argument's type (L6-01..07/13/16/30 pass); match-result
slots zero-init pointer/double as `null`/`0.0` (clang rejects
`store i8* 0` / `store double 0`; L2-12/14/15/16 build); the script cache
key now includes the compiler build identity (a new build used to serve the
old build's binaries); `xiom fmt|lsp|mcp|pkg|dbg|verify|ffigen` dispatch
natively via the sibling binary; release.yml ships `xiom-wasm-<ver>.wasm`
plus `bin/xiom-wasm.wasm` (C8). Lock `e2e_m92_interface_dispatch_zero_init`.
OPEN with repros in docs/COMPILER_BUGS.md R48: L6-28/L6-40 (interface T from
struct generic), L5-40 (nested-generic mono name), L6-31 (qualified ctor
redefinition), L8-15/18 (i64 passed as ptr), L6-05/L2-19 runtime AVs, 26
C18/C19-residue lessons, and the fmt-closure perf item (reachable-function
peek, deferred to Stage 6).

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
2. **clap migration -- DONE (2026-09-18)**: `crates/xiom/src/cli.rs` is the
   parser of record (53 boolean, 3 optional-value, 18 required-value, `-o`,
   `-g`, positionals; lenient `ignore_errors`; help/version auto-flags
   disabled) and now returns a `Cli` whose matches drive the main-path reads
   (`flag`/`value`/`values`/`present`), while `Deref` to the original argv
   keeps the early dispatch, `run`'s mini-language, command words and the
   `--flag=value`-sensitive diagnostics/sandbox/graph checks byte-identical.
   5 unit tests pin the surface, representative invocations, and the reads.
   The two flag-scanning helpers were deleted. Verified by the full e2e.
3. **fmt: body-inline comment trivia attachment -- DONE (2026-09-18)**.
   `format_source_text` threads lexer trivia (comments) and closing-brace
   token positions into the Formatter: leading comments emit above the
   statement/decl, same-line comments stay trailing, comments before a
   block's `}` stay inside it; expression-internal comments attach to the
   next statement or block close (never dropped). Idempotent; 3 new tests
   + real-file round-trip. Remaining fmt polish: none tracked.
4. **LSP cross-file index -- DONE (2026-09-18)**: `Backend` keeps a
   `FileIndex` (symbol -> declaration sites) built lazily from the open
   document's project roots (document dir, `src/`, graph source roots),
   cached across requests and rebuilt after didOpen/didChange/didClose.
   `textDocument/definition` now resolves declarations in files that were
   never opened (open documents still win), and `path_to_uri` round-trips
   Windows drive letters. Root discovery deliberately does NOT walk broad
   parents (a temp-dir file must not index all of /tmp); traversal is
   bounded (4000 files / depth 24, skip dirs). lsp 45/45 (new
   `test_definition_cross_file_index_unopened_file` covers index hit +
   rebuild-after-change).
5. **Driver hygiene -- DONE (2026-09-18)**: per-invocation temp names are
   randomized across the driver (watch/REPL/script siblings, jit link
   artifacts) and the JIT temp directory now mixes pid with the
   time+counter suffix and is removed after the library is dropped (pid
   reuse plus a stale dir could previously hit the same paths).
6. **cargo-vet audits -- DONE (2026-09-18)**: `cargo vet` (0.10.2) is
   bootstrapped with `supply-chain/config.toml` exempting the current 171
   crates ("safe-to-deploy"); the CI hygiene job installs cargo-vet and runs
   `cargo vet` next to cargo-deny, so any new dependency or version bump is
   unvetted until audited or explicitly exempted. Imports/audits files are
   empty until the first upstream audit import.
7. **Same-leaf TYPE collision -> FIXED (R39, 2026-09-17)**: catalog injection
   module-qualifies colliding non-generic type leaves and rewrites references
   (`type_qualify.rs`; lock m84). The next bench clang error is the generic-mono
   Pair ABI mismatch -- see "Open compiler findings".
8. **Stage 6 continuation**: real incremental engine, parallel
   monomorphization profiles, linker strategy, more budget metrics.
9. **Stage 7 selfhost ladder**: v092..v11 are milestone emitters, not yet a
   full XIOM-in-XIOM compiler; zero-ICE self-build is a multi-phase project.
   The selfhost COMPILE gate is green.
10. **Release pipeline (website + ops lanes, 2026-09-18) -- DONE**:
    `docs/COMPILER_RELEASE_BATCH.md` (xiom-lang/ops) executed. Version is
    single-source at **0.61.0** (`Cargo.toml [workspace.package]`, 20
    workspace crates + the fuzz workspace): REPL/doctor, `xiom-pkg` help,
    `xiom-dbg` help, `xiom-wasm get_version` all print
    `env!("CARGO_PKG_VERSION")`; `xiom.bat`, ascii art, MCP manifest and the
    installers/package scripts carry no literals (installers resolve
    XIOM_VERSION env -> release dir name -> workspace Cargo.toml; package
    scripts default `-Version` from the workspace and no longer rewrite
    Cargo.toml). `release.yml` has a `guard` job (tag == workspace version
    AND tag is an ancestor of main; dry runs read the version), builds
    `xiom` + `xiom-pkg`, stages both binaries in Windows/Linux/macOS
    archives, asserts the staged `--version` output matches the label, and
    the `compiler-release` dispatch (client_payload {tag, stdlib_ref,
    compiler_ref=tag}) now uses `XIOM_RELEASE_TOKEN` -- extend that PAT to
    `xiom-lang/website` (Contents: read/write) or the dispatch 404s.
    Verified locally: workspace check --all-targets, release build, both
    `--version` + doctor, pkg/dbg/wasm/xiom tests, and a local
    `package.ps1` run (archive `xiom-v0.61.0-windows-x64.zip` carries
    bin/xiom.exe + bin/xiom-pkg.exe, both self-report 0.61.0).

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
Continue the AXIOM compiler-lane campaign in E:\xiom-lang\xiom (the compiler
repo; post-split, branch `main`). Read SESSION.md -- the "Current state
(2026-09-19, post-R48)" section at the top -- and docs/COMPILER_BUGS.md R48
before touching code. Stage 5 is COMPLETE and all R-bugs through R48 are
fixed; this session's mission is to CLOSE ALL OPEN COMPILER BUGS (the R48
residue) with production-grade fixes, not to start new features.

Stdlib: clone `xiom-lang/stdlib` at `STDLIB_VERSION` into `stdlib/` or set
XIOM_STDLIB before any stdlib/smoke test; everything resolves through
`xiom_graph::paths` (CI uses XIOM_REQUIRE_STDLIB=1 with the pin checked
out). Never commit the `stdlib/` checkout.

State (verified on 56b6e0e3): e2e 2340/2340, feature-reg 510/510, checker
195/195, robustness 63/63, fuzz 24/24, perf/determinism 2/2, fmt 86/86,
lsp 45/45, stdlib-exec 85/85 (+2 ignored), xiom lib 32/32, pkg/dbg/mcp
63/34/39, cargo deny + cargo vet (175 exempted), ASCII guard, bench IR
5,808,645 bytes + `clang -c` exit 0. Workspace version 0.61.0. release.yml
ships all nine tools + pinned z3 + the wasm asset; the tag==version/on-main
guard and the compiler-release docs dispatch are wired. Git is clean and
pushed.

OPEN BUGS (docs/COMPILER_BUGS.md R48 has exact repros -- reproduce each
BEFORE fixing):
1. C17 interface residue -- L6-28 (`PriorityQueue[T: Priority].insert`:
   T must infer from the struct's generic), L6-40 (module-scoped
   `Runner[T: Plugin]` dispatch). Both still fail codegen with
   `C001: type 'Int' does not implement ...`.
2. C17 clang residue -- L5-40 (`alloca %struct.Option__Vec_Str_` nested
   generic mono name never defined), L6-31 (`invalid redefinition of
   function 'school.students.new_student'`, the C11 qualified nested
   constructor emitted twice), L8-15/L8-18
   (`%tmp defined with type 'i64' but expected 'ptr'` -- a Vec receiver
   reaches `get_Int(%struct.Vec* ...)` as an i64 handle).
3. C17 runtime access violations -- L6-05 and L2-19 build but crash with
   `0xC0000005` at run.
4. C18/C19 residue (26 lessons) -- 19 nondeterministic pointer prints
   (L0-11, L3-01/02/07/50, L5-02/05/07/09/20/24/26/29/31/35/36/43, L8-14,
   L8-20), 7 invalid-UTF-8 outputs (L0-34, L0-49, L0-50, L5-32, L5-34,
   L5-42, L7-09), L5-21 (Float64 `.to_str()` inside a generic over
   `Vec[T]` still prints IEEE bit patterns).
5. Perf -- the auto-injected `xiom.fmt` closure costs +2.3-2.5 s on the
   first `.to_str()` compile. Implement the reachable-function-only peek in
   `xiom-check::collect_external_decls` (peek the checker-resolved module
   shallow, run the reachability filter, then pull the deps named by the
   SELECTED decls to a fixpoint) and gate it with the full e2e.
6. C3 (script-mode flags) and C6 (stdlib `package.xi`): request the exact
   playground repro before changing semantics; document if deferred.

Repro recipe: `git clone --depth 1 https://github.com/xiom-lang/playground
tmp/playground`; every `lessons/**/Lx-yy.json` has a `.solution` field --
extract it to `tmp/lessons/Lx-yy.xi` and run `target\debug\xiom.exe -o
tmp\lessons\Lx-yy.exe tmp\lessons\Lx-yy.xi` (run `cargo build -p xiom`
after any checker/codegen change first). Determinism check: run the exe
twice and compare stdout hashes.

Method: reproduce -> minimize to a fixture under `tests/regression/m93_*`
-> fix with the smallest correct change -> add an e2e lock + the CI lock
line -> mark the finding FIXED in docs/COMPILER_BUGS.md (with the repro and
the evidence) -> update SESSION.md -> atomic commit (code + docs + locks +
CI lock line together). Drop temporary debug prints before committing.

Verification commands: `cargo test -p xiom-codegen --test e2e_tests`
(2340+, ~25 min; the stdlib-dependent tests need the checkout);
`--test feature_regression_tests` (510), `-p xiom-check --lib` (195),
`--test perf_budget_tests` (determinism canary + budgets),
`--test robustness_tests` (63), `--test fuzz_tests` (24),
`--test stdlib_execution_tests` (85 +2 ignored), `cargo test -p xiom --lib`
(32), `cargo test -p xiom-pkg -p xiom-dbg -p xiom-lsp -p xiom-mcp`,
`cargo vet`, `cargo deny check advisories licenses bans sources`, IR gate:
`xiom --emit-ir examples\benchmark\main.xi > b.ll; clang -c b.ll -o NUL`
must exit 0. Capture $LASTEXITCODE immediately after every native command.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Use --emit-ir /
clang -c / --sanitize=address for miscompile work. Never commit `stdlib/**`
or the `stdlib/` checkout. Conventional commits, atomic slices. The whole
suite must be green before each commit; a full e2e takes ~25 min, so batch
related fixes per run and always re-run it after a codegen change.
```

