# XIOM Handoff -- 2026-09-16 (compiler lane; rounds 61-76 in docs/SESSION.md)

Branch `feat/architect`. Last compiler commit: the round-76 slice (perf
budgets + partial R25 determinism fixes); the supply-chain signing slice
(`eb5a5536`) precedes it. Working tree should be clean except the generated
`.xiom_ai.json` and the parallel stdlib lane's files. The stdlib session
commits to the same branch; NEVER stage their `stdlib/**`,
`examples/stdlib_smoke/**`, `docs/stdlib_session.md`,
`docs/STDLIB_READINESS_PLAN.md`, `docs/STDLIB_DEDUP_INVENTORY.md`. They also
sometimes sweep the whole tree into their commits (it happened twice: my
DWARF work landed inside 82d66b98), so re-check `git log --stat` if a change
seems missing.

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
- **Stage 6 start + R25 PARTIAL (round 76)**: `perf_budget_tests.rs` (IR
  byte budgets + 180s ceiling + byte-identical determinism canary), CI-wired;
  deterministic variant->parent-enum and `type_meta` selection
  (`pick_deterministic`: current module -> shortest key -> lexicographic).

Last full gates (round 76): checker 189/189, stdlib-exec 85/85 (+2 ign),
feature-reg 510/510, e2e **2328/2328**, perf 2/2, lsp 44/44, dbg 34/34,
mcp 39/39, pkg 52/52, workspace `--all-targets` clean, selfhost v092
compiles.

## Immediate task: R25 -- deterministic fn-REFERENCE resolution

The perf determinism canary still fails on the 30-module bench graph (sizes
drift ~130-230 bytes across runs). Exact evidence (round-76 notes in
docs/SESSION.md): inside `benchmark.collections.test_partition()` the fn
value coercion emits

```
run A: %tmp38 = ptrtoint i64 (i64)* @benchmark.comptime.is_even to i64
run B: %tmp38 = ptrtoint i64 (i64)* @benchmark.math.is_even    to i64
```

Two same-named free fns in different modules; the pick is HashMap-ordered.
Find the fn-ref coercion path (`is_fn_ref`, `bare_fn_aliases`,
`functions.keys().find`, `wrap_fn_ref_env` call sites in call.rs /
vec_abi.rs) and apply scope-first + `pick_deterministic`-style ordering.
Verify with:

```
target\debug\xiom.exe --emit-ir examples\benchmark\main.xi > a.txt   (x3, compare bytes)
cargo test -p xiom-codegen --test perf_budget_tests
```

Then tighten `perf_determinism_ir_is_byte_identical` to include the bench
graph. Add the formal R25 entry to docs/COMPILER_BUGS.md once that file is
clean (it had stdlib-lane WIP at handoff; do not sweep their edits).

## Remaining queue

1. **R25** (above).
2. **Supply-chain tail**: transitive dependency closure from registry
   metadata; server-side publish authentication.
3. **fmt**: body-inline comment trivia attachment (stage-2 trivia
   dependency; shebang/header/string escaping already done round 42).
4. **cargo-vet audits** (cargo-deny already runs in CI).
5. **clap migration** of the driver parser (large; keep the CLI surface
   byte-compatible and gate with the full e2e suite).
6. **Stage 7 selfhost ladder**: v092..v11 are milestone emitters, not yet a
   full XIOM-in-XIOM compiler; zero-ICE self-build is a multi-phase project.
   The selfhost COMPILE gate is green.
7. **Stage 6 continuation**: parallel monomorphization profiles, linker
   strategy, more budget metrics.

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
- Never edit `stdlib/**`; report stdlib findings in `docs/ITEM_A_STDLIB_FINDINGS.md`
  / COMPILER_BUGS and in chat.

## Paste-ready prompt for the next compiler session

```
Continue the AXIOM compiler-lane readiness campaign in E:\Projects\AXIOM on
branch feat/architect. Read SESSION.md (repo root) and docs/SESSION.md
(rounds 61-76; round 76 has the R25 evidence) before touching code. The
stdlib session works in parallel on stdlib/** only and commits to the same
branch (they sometimes sweep the whole tree -- re-check git log if a change
seems missing); never stage their files.

State: Stage 3 Item A CLOSED (strict catalog findings, checker 189/189),
R-bugs through R24 CLEARED with locks m74-m80, e2e 2328/2328, supply chain
signed (ed25519 keygen/trust/sign/verify, fail-closed installs, ureq-only
publish, git commit pins), Stage 6 perf budgets wired, selfhost v092 compile
gate GREEN.

Your task, in order:
1. R25: make fn-REFERENCE resolution deterministic. Evidence: the bench
   graph emits `ptrtoint @benchmark.comptime.is_even` vs
   `@benchmark.math.is_even` across runs inside
   benchmark.collections.test_partition(); find the fn-ref coercion path
   (is_fn_ref / bare_fn_aliases / wrap_fn_ref_env call sites) and apply
   scope-first + deterministic key ordering (pick_deterministic pattern).
   Verify with 3x `--emit-ir examples\benchmark\main.xi` byte compares and
   `cargo test -p xiom-codegen --test perf_budget_tests`; then extend the
   determinism test to the bench graph and add the R25 entry to
   docs/COMPILER_BUGS.md (only when that file is clean of stdlib-lane WIP).
2. Supply-chain tail: transitive dependency closure from registry metadata,
   server-side publish authentication; then fmt body-inline comment trivia,
   cargo-vet, clap migration.
3. Stage 7 selfhost ladder (v092..v11 are emitters; the full self-build is a
   multi-phase project) and Stage 6 continuation (parallel mono profiles,
   linker strategy).

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Capture $LASTEXITCODE
right after each native command. Use --emit-ir / --sanitize=address for
miscompile work. Keep commits atomic (code + docs together) and never touch
stdlib/**.
```
