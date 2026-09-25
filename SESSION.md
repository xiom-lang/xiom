<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# CONTINUATION HANDOFF (2026-09-24, compiler lane)

Supersedes the 2026-09-23 header. The 2026-09-16 handoff and the dated
timeline below are the detailed log -- read the newest entries for evidence.

## Status snapshot

- **Released:** compiler **v0.61.3** (release run 35889350771, all platforms +
  VSIX 0.12.0 live on both marketplaces). Compiler matches stdlib v0.61.3;
  `STDLIB_VERSION` pins `stdlib-v0.61.3`.
- **Git:** `origin/main` = `1f7a9fbc` (R66 + its handoff note are PUSHED).
  Local `main` carries the whole campaign ahead of origin: R67+R68, R69, R70,
  R71, R72/m127, m125 (freeze + CI gate), m126 (deterministic publish bytes +
  `publish --tarball`), m128 (publish sends the registry `compiler` field),
  the front-end audit (`docs/FRONTEND_AUDIT.md`), the website release-notes
  publisher (`crates/xiom-release-notes`), the docs/handoff commits, and the
  Sprint A front-end batch (shared toolchain probe + doctor v2 + FE-8/FE-9).
  Tree clean.
- **stdlib checkout:** `stdlib/` is now **detached at the pin**
  `stdlib-v0.61.3` (`c7b4027`). It had been 6 commits stale (`385e1e4`);
  Phase 0 refreshed it and re-ran the stdlib-dependent suites on the pin.
- **No red gates.**

## Gates (latest evidence)

- Full e2e **2375/2375 (+4 ignored)** -- re-run ON THE PIN for the
  non-blocker batch (2026-09-24, 1809.8 s; the 4 ignored are the relocated
  benchmark-chaos runs). The relay batch ran 2379/2379 (1871.5 s); Sprint D
  2377/2377 (1449.5 s); Sprint C 2377/2377 (1512.3 s); Sprint B 2374/2374
  (1691.6 s); Sprint A 2373/2373 (1851.5 s).
- checkers: checker 195/195, parser 102/102, integration 130/130,
  feature-reg 510/510, robustness 63/63, fuzz 24/24.
- stdlib-dependent (re-run on the pin): api-freeze **2/2** (m125 regen),
  stdlib-exec **85/85 (+2 ignored)**.
- tooling: pkg **75/75** (m126 + m128 + FE-9 alias), release-notes **6/6**
  (new), mcp 39/39, ascii_guard OK; new front-end locks: toolchain 7/7 +
  doctor 10/10 unit tests and `doctor_cli` 2/2 (CI step "Front-end CLI
  lock").
- pre-flight for the combined release (2026-09-24, on the current pin):
  workspace lib scan clean, integration 130/130, feature-reg 510/510,
  robustness 63/63, fuzz 24/24, api-freeze 2/2, stdlib-exec 85/85 (+2
  ignored), stdlib modules 40/40, mcp 44/44, pkg 75/75, release-notes 6/6,
  perf 3/3; release-notes/v0.62.0.md+json drafted and `verify`-green
  (fragment-free; re-convert after the pin bump).
- CI: ubuntu-latest green on the pushed R66 state. The Windows CI leg has NOT
  been re-run since R66 (pushes to main do not trigger CI) -- the next push/PR
  validates it; R66 removed the previous Windows-only failure.

## Owner decision: ONE combined release (no intermediate tags)

Git free-plan build minutes are charged, so everything ships in a single tag.
Recommended version: **v0.62.0** (user-visible behavior + UX changes).
Ordering:

1. Compiler lane completes **Sprint A (front-end P0)**, **Sprint B (P1)** and
   **Sprint C (fn-value / generic-mono ABI + E001)**, each batch with its lock
   + one full e2e, committed/pushed as we go (pushes are free).
2. **stdlib lane** completes its 100% readiness plan (coverage waves, dedup
   translation units, tzdata phase 2, untested-surface classes, stub
   re-triage), adds `release-notes/<tag>.md` for the new tag, and cuts ITS
   release.
3. Compiler bumps `STDLIB_VERSION` to the new stdlib tag, runs the full gate
   list on the pin, bumps the workspace version to `0.62.0`, authors
   `release-notes/v0.62.0.md` (the release workflow's `verify` gate REQUIRES
   the committed JSON before the tag), pushes, and tags.
4. **Registry** re-canary after the tag. **Website lane** publishes the notes
   JSON to `dl.xiom-lang.org/releases/<tag>/release.json` and sets
   `"notes": true` in `releases/index.json` (the dispatch now carries
   `client_payload[notes_path]`).

TLS/schannel FFI hardening (stdlib B-list, gates any HTTPS/TLS claim) must
ship in this same window or be explicitly excluded from the release notes.

## Queue, in order

**Sprint A -- front-end P0: LANDED (2026-09-24)** -- see the "Sprint A
landed" note above. FE-1..FE-5, FE-7, FE-8, FE-9 complete (+FE-16 via the
embedded pin); FE-6 (`doctor --deep`) deliberately deferred to Sprint B.

**Sprint B -- front-end P1: LANDED (2026-09-24)** -- see the "Sprint B
landed" note above (FE-6, FE-10..FE-15, FE-17 complete).

**Sprint C -- fn-value / generic-mono ABI unification** (highest correctness
item; one batch or split by root cause, each with locks):
- m127 residuals: `Vec[fn].new()+push` elements, element-to-local calls,
  fn-element calls through struct fields (repros `tmp/probe_p1/`).
- packages lane's 3 confirmed probes (`tmp/fp_probe/`, originals under
  `%TEMP%\kilo\fp-probe\`): fn-ptr struct field (clang ptr/i64 mismatch),
  generic `[T,U]` fn-ptr with U=Str (corrupt Str), generic `Vec[U]` map
  (mis-written elements; AV for U=Str).
- stdlib lane's cross-type callback matrix (their `tools/known_failures/`):
  cross-type returns wrong (Int->Str by-ref/by-value, Int->Float64,
  `sort_by_key[Int,Str]`); same-type and concrete correct; Str->Int correct.
  Root cause: the call/return ABI must follow the MONOMORPHISED
  instantiation, not the erased generic signature.
- E001 conservatism: no warning for consumed temporary borrows; never weaken
  the genuine-overlap warning (their
  `tools/probes/evidence/p_e001_borrow_conservatism.xi`).
Specs: `docs/COMPILER_BUGS.md` (m127, R66-R72) + the 2026-09-24 timeline
entries.

**Sprint D**: LANDED (2026-09-24) -- `xiom toolchain check [--json]` and the
MCP `get_contracts`/`search_symbols` tools shipped (see "Sprint D landed").
The `update`/`rollback` half still needs in-process provenance-attestation
verification (dependency procurement first); it refuses with exit 3 and the
status is recorded in `docs/POST_RELEASE_PLAN.md` section 1.

**Sprint E**: Stage 6 (performance program) and Stage 7 (selfhost gate).
**Stage 5 is CLEAR** (compiler lane). Starting points, from the Stage 6/7
notes: Stage 6 has its gate (`crates/xiom-codegen/tests/perf_budget_tests.rs`:
IR byte budgets + 180s ceiling + byte-identical determinism canary, CI-wired)
and one landed step (deterministic `pick_deterministic` selection); its
continuation items are the real incremental engine (only reachable functions
re-emitted, per-body snapshot cache) and parallel codegen.
**Stage 6 also has a queued work order now: the control-flow lint wave --
`docs/STAGE6_LINT_WAVE.md` (W002 unconditional recursive cycle from the
2026-09-25 owner playground probe, W003 unreachable-after-divergence, W004
unreachable match arm, tier-2 W005-W007; warning-only, user-program scope).**
Stage 7 is the
selfhost ladder (v092..v11 milestone emitters, not yet a self-compiling
chain). Both are multi-session programs; scope the first batch from the
Stage 6 notes before touching code.

**Small cleanups** (from older handoffs): DONE (2026-09-24) -- the
`xiom-benchmark-chaos` e2e tests t2-t5 no longer point at the removed
monorepo checkout (relocated to the internal `tests/ecosystem/t*.xi` copies,
ignored with a reason; compile coverage stays in `e2e_i2_parallel_codegen`),
and the e2e harness deletes its per-invocation outputs (binary + .wasm).
Legacy-package migration note/codemod remains the packages lane's policy.

## Cross-lane status

- **stdlib:** m125 freeze regen CLOSED independently on their side (2/2 on
  compiler main). They re-probed R67/R68/R69/R70/R72 and confirmed the
  still-open `Vec[fn]` literal / `.new()+push` AVs; their green lock
  `p_r70_pending_shapes.xi` is parked for the pin-bump commit. They still owe:
  the full readiness plan, a `release-notes/<tag>.md` fragment for the next
  release, and (optional) minimal repros for checker E001 + the generic
  fn-pointer limitation (now delivered for the latter).
- **registry:** policy settled at their `5c1cdcb`: deterministic assets make
  re-runs byte-stable; re-canary after any asset regeneration or packing
  change. m126 delivered the client-side half of exact-bytes promotion
  (deterministic writer + `publish --tarball`, printed SHA256) and m128 now
  sends the per-version `compiler` field (manifest `compiler:` or
  `--compiler <tag>`; warns when absent) so the website can correlate
  packages with toolchain releases. Nothing pending from ops; production
  publish waits on the owner's environment approval. Open: FE-9 name
  alignment (`xiom.std` vs `xiom-std`); package repos should add `compiler:`
  to their manifests.
- **packages:** 3 repros confirmed on our build; they will commit the probes
  under `docs/repro/`. Their migration-note request (old dialect) is policy,
  not a parser bug; the `xiom.ffi` triage abort is harness-side.
- **website:** schema-v1 contract implemented on our side (publisher + gate +
  CI tests); requirement 5 (dl + `releases/index.json` `notes: true`) is
  their lane; the dispatch now carries `notes_path`.
  - Playground UX observation (2026-09-25, owner probe): a program that
    crashes (e.g. infinite mutual recursion -> stack overflow, exit
    `0xC00000FD`) is reported as "Program ran with no output". The runner
    should surface the non-zero/crash exit instead of only the empty stdout.
    Not a compiler bug (repro in the relay below); website-lane fix.
- **benchmark:** `COMPILER_VERSION` still pins the absent v0.61.0 -- they
  should bump after our next release exists.

## Environment / method notes

- **Identity**: gh active account must be **Lefteris-Notas**
  (`gh auth switch --user Lefteris-Notas`). Repo-local git identity is
  `Lefteris Notas <lefterisnotas@gmail.com>` (global differs; leave it).
- **CI does NOT run on pushes to main** (PR + `workflow_dispatch` only).
- **Never rebuild `target/debug/xiom.exe` while a cargo e2e runs.** One e2e
  is ~23-28 min; batch fixes per run.
- **`--emit-ir` prints INTERMEDIATE output.** For the IR clang compiles,
  force a link failure (`--link missing_xyz`) and read `<output>.ll`.
- **Large fixed arrays** (>= 16 KiB) are memset-zeroed and accessed by
  address (`LARGE_ARRAY_MIN_BYTES`); clang 22 ISel crashes above 32 KiB.
- Repro harnesses: `tmp/repro_lessons.ps1`, `tmp/output_check.ps1`;
  playground clone `tmp/playground`.
- Stdlib override for a NEWER stdlib: set BOTH `XIOM_STDLIB` and
  `XIOM_RUNTIME_DIR` (runtime C resolves separately).
- **Test paths must use `/`**; the e2e stdlib guard accepts both layouts.
- WSL/Linux CI proxy: `XIOM_REQUIRE_STDLIB=1` + the curated `ci.yml` list.
- Bench harness on Windows: set a clean `TMP`/`TEMP` first.
- **ascii_guard**: staged files must be pure ASCII
  (`python tools/ascii_guard.py check`).
- Delete extracted release archives after verification (stale stdlib trees
  trip `e2e_m17_zero_warnings`).
- ISel/LLVM minimizer: `tmp/extract_closure.py` + `tmp/preprobe/`.
- Local repro dirs: `tmp/probe_p1/` (R66-R72 + m127), `tmp/fp_probe/`
  (packages fp probes), `tmp/relnotes_smoke/` (release-notes CLI smoke).

## Continuation prompt (copy/paste into the next session)

> Continue the XIOM compiler-lane campaign in `E:\xiom-lang\xiom` (branch
> `main`; push only when asked). Read the top section of SESSION.md
> ("CONTINUATION HANDOFF (2026-09-24, compiler lane)") plus
> `docs/FRONTEND_AUDIT.md` (the owner-approved P0/P1 backlog) and
> `docs/COMPILER_BUGS.md` (R66-R72, m127, Sprints A-D) before touching code.
>
> State: compiler v0.61.3 released; `STDLIB_VERSION` pins `stdlib-v0.61.3`
> (the local `stdlib/` checkout is detached at that tag); `origin/main` has
> R66 only, with the whole campaign unpushed on local `main` (R67-R72, m125,
> m126, m128, the audit, the release-notes publisher, Sprints A-D, docs).
> Full e2e 2375/2375 (+4 ignored) on the pin (non-blocker batch); api-freeze
> 2/2 and stdlib-exec 85/85 on the pin; no red gates. The owner
> batched everything into ONE release (no intermediate tags; recommend
> v0.62.0), sequenced as: compiler Sprints A+B+C -> stdlib completes its plan
> + cuts its release + ships its release-notes fragment -> bump
> STDLIB_VERSION -> full gates on the pin -> author release-notes/v0.62.0.md
> (the release workflow's `verify` gate requires the committed JSON BEFORE
> the tag) -> version bump + push + tag -> registry re-canary, website
> publishes the notes to dl.
>
> Sprints A-D are LANDED -- see "Sprint D landed" above; do not redo them.
> OWNER SEQUENCE (2026-09-24): ship the combined release BEFORE Stage 6.
> The compiler side is release-ready -- see "Release pre-flight" above; it
> waits only on the stdlib lane (their plan -> their release with the
> release-notes/v0.62.0.md fragment -> the ref). If the pin has landed, run
> the release sequence (bump STDLIB_VERSION -> stdlib-dependent gates on the
> new pin -> re-convert notes -> version bump -> push -> tag -> canary);
> otherwise start Stage 6's non-blocking prep. Stage 5 is CLEAR. The Stage 6
> gate exists at `crates/xiom-codegen/tests/perf_budget_tests.rs`; the real
> incremental engine + parallel codegen and the Stage 7 selfhost ladder are
> the open work.
>
> Method (non-negotiable): work repro-first; rebuild `cargo build -p xiom`
> after checker/codegen changes; add an e2e lock (`e2e_mNNN_*` fixture + the
> CI lock line in `.github/workflows/ci.yml`) for every compiler behavior
> change; run the full e2e ONCE per batch; never rebuild while an e2e runs;
> keep `python tools/ascii_guard.py check` green; commit atomically with
> evidence in SESSION.md and docs/COMPILER_BUGS.md.
>
> After Sprint E: the release sequence in the State paragraph (stdlib
> completes -> pin bump -> full gates -> release-notes/v0.62.0.md -> version
> bump + push + tag -> registry re-canary, website notes to dl).

## Sprint A landed (2026-09-24, compiler lane)

Front-end P0/P1 (FE-1..FE-5, FE-7, FE-8, FE-9; FE-16 via the pin embed) is
implemented and gated. Details + evidence: docs/COMPILER_BUGS.md
("2026-09-24 -- Front-end audit Sprint A").

- `crates/xiom/src/toolchain.rs`: ONE clang/opt/nasm probe for doctor AND the
  driver (PATH first, resolved to absolute paths; then per-OS known locations
  incl. the winget globs; the personal NASM path and `find_tool`/`find_nasm`
  are gone; `opt` follows clang's directory).
- `crates/xiom/src/doctor.rs`: doctor v2 -- identity block (exe/install root/
  XIOM_HOME/stdlib root via `paths::stdlib_root()` + version/tool path+version/
  runtime), warnings (compiler<->stdlib version, stdlib<->embedded pin,
  runtime missing, duplicate installs), OS-specific remediation with no
  `xiom install llvm`, `--json`, exit codes 0/1/2.
- FE-8: `xiom publish` is a deprecation alias for `xiom pkg publish` (legacy
  git-tag flow deleted); `xiom update` points at the reinstall one-liner /
  future `xiom toolchain update`; pkg help example fixed; the old improvement
  plan is marked historical.
- FE-9: `xiom.std` <-> `xiom-std` client aliases; live-verified (`info` both
  spellings -> v0.61.3 sha256 1ad1b33a5caa; `install xiom.hello` end-to-end
  into a temp XIOM_HOME).
- Locks: 7 toolchain + 10 doctor unit tests; `doctor_cli` 2 binary tests
  (CI step "Front-end CLI lock"). Workspace all-targets check clean; ascii
  guard clean; **full e2e 2373/2373 on the pin**.
- Deferred: clang version floor (no agreed floor), `doctor --deep`/`--fix`
  (Sprint B FE-6), installers FE-10..FE-15, grouped help FE-12.

Next: Sprint C (fn-value/generic-mono), then Sprint D, E.
(Landed -- see "Sprint C landed" below.)

## Sprint B landed (2026-09-24, compiler lane)

Front-end P1 (FE-6, FE-10..FE-15, FE-17) is implemented and gated; details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Front-end audit Sprint B").

- FE-17: one optional `;` after a `module` header (brace-less and block form)
  is tolerated; lock `e2e_m129_module_trailing_semicolon` (+CI line) and a
  parser unit test.
- FE-6: `xiom doctor --deep` compiles AND RUNS a trivial program end-to-end.
- FE-12: grouped `xiom --help` (no sprint tags; every flag kept; tool help
  line for the dispatcher).
- FE-13: `xiom --version` prints the install root; `xiom.bat` prefers its own
  directory over a stale LOCALAPPDATA install.
- FE-10/FE-14: installers ship all 9 tools + the archive `lib/` layout and
  finish with `xiom doctor` (`--json` in CI); modern banners. Found and fixed
  a pre-existing `-Unattended` crash (`$registerExt` vs the `$RegisterExt`
  switch parameter).
- FE-15: the generated uninstaller cleans PATH (user + best-effort machine).
- FE-11: unpinned LLVM direct downloads removed from install_deps.ps1/.sh.
- Gates: parser 102/102; full e2e **2374/2374** on the pin; workspace
  all-targets check clean; ascii guard clean.

## Sprint C landed (2026-09-24, compiler lane)

fn-value / generic-mono ABI unification + E001 conservatism; full details and
root causes in docs/COMPILER_BUGS.md ("2026-09-24 -- Sprint C").

- Parser/checker: `Vec[fn() -> Int]` type args keep the fn marker (no more
  `Ident("_")`); `method_target` distinguishes `Type.method[Arg]()` /
  `module.fn[Arg]()` from `value.field[i]()`; array literals of fn refs type
  as `Vec[fn(...) -> R]`.
- Codegen: fn markers in `type_string_full`/`type_annotation_name`, i64-erased
  fn locals, fn-ref markers recorded for array bindings, for-loop fn elements
  called env-first, raw array buffers wrap fn refs, `(op.f)(x)` unwraps to the
  env-first field path.
- Generic mono: fn-typed params infer T/U from the argument's registered
  signature (`fn_arg_generic_binding` + LLVM->XIOM name mapping) -- cross-type
  U=Str / U=Float64 callbacks and `Vec[U]` returns are correct.
- E001: per-statement temporary-borrow release + the `read_borrow_count`
  sentinel fix; `smoke_collect_sparse` 7 -> 0 warnings, genuine overlap still
  warns.
- Locks: e2e m130 (fn-value ABI), m131 (generic callback ABI), m132 (E001
  consumed); `crates/xiom/tests/borrow_e001.rs` (m132/m133 warning contract,
  CI step extended). Probes: 15 probe_p1 + 7 fp + stdlib matrix all green.
- Gates: full e2e **2377/2377** on the pin (1512.3 s); checker 195/195;
  parser 102/102; workspace all-targets clean.
- Open follow-up recorded: the checker accepts a by-ref callback where a
  fn-typed param declares `fn(T) -> U` by value (runtime AV); needs a
  fn-signature compatibility check.

## Sprint D landed (2026-09-24, compiler lane)

`xiom toolchain check` + the MCP structured-contract tools; details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Sprint D") and the status note in
docs/POST_RELEASE_PLAN.md section 1.

- `crates/xiom/src/toolchain_cmd.rs`: GitHub-Releases-only `check [--json]`
  (`current/latest/platform/up_to_date/notes` + `install_kind/asset/exe`),
  exit codes 0/1/2, package-manager + dev install detection; verified live
  (v0.61.3 = latest, exit 0). `xiom doctor` points at the command.
- `update`/`rollback` refuse with exit 3 pending in-process attestation
  verification (dependency procurement); the refusal names `check` + the
  release installer. POST_RELEASE_PLAN records the block precisely.
- MCP: `get_contracts {symbol, verify?, file?}` (structured signature +
  requires/ensures/invariants with source lines; Z3 fold when `verify:true`)
  and `search_symbols {query, file?}` (ranked stdlib + project hits), built on
  the same live stdlib catalog scan as `xiom_stdlib_reference`. 18 tools.
- Gates: full e2e **2377/2377** on the pin (1449.5 s); MCP 44/44 (5 new
  contract tests); workspace all-targets clean; live check green.

Next: Sprint E (Stage 6/7).

## Relay batch landed (2026-09-24, compiler lane)

Two stdlib-relayed compiler findings fixed (the third, the cross-type
callback matrix + E001 repro, landed in Sprint C); full details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Relay").

- Private same-leaf type collision (R44/Timer case): the collision triage now
  includes private type decls, so `xiom.async.Timer` (private) and
  `xiom.async.timer.Timer` (pub) emit distinct qualified layouts instead of
  silently sharing one. Lock `e2e_m134_private_same_leaf`.
- Clause-position `Bool == Int`: clauses were never type-checked; a light
  clause validator now rejects Bool-vs-concrete comparisons (plus an
  `@pre` typing arm). Locks `checker_locks.rs` (m135 reject, m136 green) +
  `e2e_m136_well_typed_clauses`.
- Deferred with precise evidence (needs stdlib-lane clause fixes before the
  compiler can require every clause to be Bool): the 8 stdlib clause sites
  listed in COMPILER_BUGS. Also recorded: `@pre` on a method call has a wrong
  RUNTIME snapshot (typing-only lock).
- Gates: full e2e **2379/2379** on the pin (1871.5 s); checker 195/195;
  catalog corpus clean; workspace all-targets clean.

## Stage 6 increment (2026-09-24, compiler lane)

Sprint E started with the smallest safe bite: the fmt-peek closure shape now
has its own perf budget. `tests/perf/fmt_peek.xi` (Int/Float64 `.to_str()`,
the shape that pulls `xiom.fmt` through `collect_external_decls`'s peek) is
covered by `perf_budget_tests.rs::perf_budget_fmt_peek_shape`: byte budget
95,000 (baseline 80,300) plus a 20 s debug-profile ceiling, so the documented
sweep p50 3.9 -> 7.9 s regression cannot grow unnoticed while the real fix
waits. Perf 3/3.

Stage 6 open work order: (1) the control-flow lint wave
(`docs/STAGE6_LINT_WAVE.md`) -- W002 unconditional recursive cycle (the
2026-09-25 owner playground probe: `a() { b(); print; } b() { a(); print; }`
never prints and dies via the runtime fault trap, exit `0xC000001D`; the
playground reporting "ran with no output" is a website-lane fix), W003
unreachable statement after a diverging statement, W004 unreachable match
arm, then tier 2 (literal `/ 0` -- today it traps at runtime; `1 << 64` --
today runs with a garbage result; non-float self-comparison); warning-only,
user-program scope, two locks per lint. (2) the reachable-function-only peek
restructure (fix shape in COMPILER_BUGS: peek the checker-resolved module
shallow, run the reachability filter, then pull the deps named by the
SELECTED decls to a fixpoint). (3) parallel monomorphization profiles,
linker strategy, more budget metrics. Stage 7: the selfhost ladder
(multi-phase).

## Release pre-flight (2026-09-24, compiler lane)

The compiler side is release-ready on the current pin; the only external
dependency left is the stdlib lane's own release.

- Gate battery re-run on the pin after all of Sprints A-D + the relay batch:
  full e2e **2375/2375 (+4 ignored)** on the final tree, checker 195/195,
  parser 102/102, integration
  130/130, feature-reg 510/510, robustness 63/63, fuzz 24/24, api-freeze
  2/2, stdlib-exec 85/85 (+2 ignored), stdlib modules 40/40, mcp 44/44,
  pkg 75/75, release-notes 6/6, perf 3/3; workspace all-targets clean.
- Compiler-side release notes DRAFTED and committed:
  `release-notes/v0.62.0.md` + `.json` (4 highlights, 3 breaking, 2 docs);
  `xiom-release-notes verify --tag v0.62.0` is green fragment-free. The
  stdlib fragment in the JSON is re-converted after the pin bump.
- Remaining sequence (blocked on the stdlib lane): stdlib completes its plan
  -> cuts its release carrying `release-notes/v0.62.0.md` -> hand over the
  ref -> bump `STDLIB_VERSION` -> re-run the stdlib-dependent gates on the
  new pin (api-freeze, stdlib-exec, stdlib modules, full e2e) -> re-convert
  the notes -> version bump 0.61.3 -> 0.62.0 -> push origin/main (FIRST CI
  run since R66 -- the Windows leg is the main unknown) -> tag v0.62.0
  (guard: tag == workspace version + ancestor of main) -> registry
  re-canary -> website publishes the notes to dl with notes: true.
- Not release-blocking: the attested updater (D2 tail, procurement) and the
  legacy-package migration note/codemod (packages lane policy). The earlier
  follow-ups (strict clause default, `@pre`-on-call runtime, private enums,
  e2e/harness cleanups) are landed or switched -- see "Non-blocker batch
  landed".
- Cross-lane update (2026-09-24 evening, stdlib relay): their clause cleanup
  landed (`Rc`/`Arc.new` -> `result.strong_count() == 1`, `ptr.replace` ->
  `dest != null`, `math.pow`/`pow_pure` `as Float64`, `array.is_sorted_by`
  implemented), and STRICT MODE is verified GREEN on their main with our
  built compiler (`XIOM_STRICT_CLAUSES=1 ... catalog_corpus_is_clean` -> 1
  passed). Their notes fragment was trimmed to 2 highlights (our 4 + their 2
  = the schema max of 6). Their release cut is gated on reaching 100%
  (coverage waves, control_theory + lp_simplex, collect/lfu dedup, geom API
  unit, tzdata phase 2, untested-surface generator), per their
  docs/RELEASE_CHECKLIST.md.
- PIN-BUMP ORDERING NOTE: the strict-default flip must land WITH the pin
  bump -- the current `stdlib-v0.61.3` checkout still carries the 8 unfixed
  clause sites and `catalog_corpus_is_clean` indexes the repo `stdlib/`
  checkout directly (it does NOT honor XIOM_STDLIB), so flipping now would
  red the corpus gate. Sequence at the bump: set `stdlib/` to their release
  ref -> bump `STDLIB_VERSION` -> `XIOM_STRICT_CLAUSES=1 cargo test -p
  xiom-check catalog_corpus_is_clean` (expect green) -> remove the env gate
  in `check_clause_bool_mix` (default becomes strict) -> re-run the
  stdlib-dependent gates + full e2e -> re-convert the notes -> version bump
  -> push -> tag.

## Non-blocker batch landed (2026-09-24, compiler lane)

The known non-blocking items are cleared or moved behind a switch; details in
docs/COMPILER_BUGS.md (relay follow-ups, status 2026-09-24 evening).

- `@pre` on a METHOD CALL -- FIXED at runtime: the pre-state collector
  recorded the callee name instead of the receiver, so `len()@pre` read the
  live length (ensures `len() == len()@pre + 1` fired although the length grew
  by exactly 1). Bare callee idents are no longer collected as variables, and
  a method with any `@pre` always snapshots `self`. m136 now runs the real
  clause at runtime.
- Full predicate-Bool clause checking -- TRANSITION SWITCH:
  `XIOM_STRICT_CLAUSES=1` enables the strict rule while the default stays
  light. All 8 stdlib clause sites were triaged as GENUINE stdlib issues (no
  checker gaps): ptr 91 undefined `old_value`, math 201/561 Float64-vs-Int,
  sync 378 / rc 29 bare `strong_count` without `()`, array 219 nonexistent
  `is_sorted_by`. The stdlib lane verifies with
  `XIOM_STRICT_CLAUSES=1 cargo test -p xiom-check catalog_corpus_is_clean`;
  when that is green, flip the default (one condition in
  `check_clause_bool_mix`). Fixture m137 + checker lock cover both modes.
- Private ENUM same-leaf triage -- landed: pub AND private enums now
  participate like types; identical layouts keep the shared key.
- Cleanups -- benchmark-chaos tests relocated to the internal copies and
  ignored (compile coverage stays in `e2e_i2_parallel_codegen`); the e2e
  harness now deletes its per-invocation outputs.
- Gates: full e2e **2375/2375 (+4 ignored)**, checker 195/195, catalog corpus
  clean, workspace all-targets clean.

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
- **CRB-6 / release dry run (2026-09-21)**: `release.yml`'s three z3 bundle
  steps now resolve the vendor license by NAME under the extraction tree
  (`find`/`Get-ChildItem -Recurse`) instead of assuming `LICENSE.txt` sits at
  the archive root -- both platform jobs had failed with `cp: cannot stat
  '/tmp/xiom-z3/LICENSE.txt'`. Committed `30c5a483`, pushed. Dispatch
  35647022547 is fully green: guard, linux-x64, windows-x64, VSIX
  (`xiom-vscode-0.12.0.vsix` + `SHA256SUMS-vscode`); marketplace publishes
  and the GitHub Release are skipped (tag-gated). No tag yet.
- **IDE distribution + release batch (2026-09-21, registry relay via owner)**:
  (1) the VS Code VSIX is now UNIVERSAL -- `program: ./xiom-dbg.exe` is gone
  from `editors/vscode/package.json`; `xiom-lsp`/`xiom-dbg` resolve at runtime
  (`xiom.debugAdapterPath` / `xiom.lsp.path` -> PATH -> workspace `target/`;
  `xiom.dbg.path` is a deprecated alias) and the bundled-binary fallback is
  removed. Verified locally with `vsce package` (9 files, 70.55 KB, no .exe).
  (2) First-run UX: activation verifies `xiom`/`xiom-lsp`/`xiom-dbg` and shows
  a notification linking to xiom-lang.org/install plus `XIOM: Recheck
  toolchain`; the toolchain is NEVER installed silently.
  (3) release.yml gained the `vscode` job: universal VSIX + SHA256SUMS ride on
  the GitHub Release; `vsce publish`/`ovsx publish` run only on tag builds and
  only when `VSCE_PAT`/`OVSX_TOKEN` are present; the release job waits on it.
  (4) `xiom --version`/`-V` always reports the workspace version; the stale
  hardcoded stats/tag defaults are gone (XIOM_RELEASE_TAG/STATS are optional
  build-time stamps) and `_build_linux.sh` no longer stamps ancient values.
  All nine tools self-report v0.61.0; the release guard + staged asserts still
  match `v<ver>`.
  (5) editors/README + vscode/README document the distribution policy;
  Visual Studio is deferred in ROADMAP M13.11; other editors stay config-only.
  (6) Marketplace metadata: `homepage` (xiom-lang.org), `bugs` (GitHub issues),
  8 keywords, dark gallery banner, and a valid 256x256 icon; release.yml now
  gates each publish step on that marketplace NOT already having the extension
  version -- bump `editors/vscode/package.json` for every extension change,
  toolchain-only releases skip publishing cleanly (vsce refuses to republish
  an existing version).
- **RELEASE v0.61.3 PUBLISHED (2026-09-23)**: compiler version now MATCHES
  stdlib-v0.61.3 (the owner asked for version parity). Run 35889350771 fully
  green: guard, windows-x64, linux-x64, macos-arm64, macos-x64, VSIX, GitHub
  Release. Assets: `SHA256SUMS`, `xiom-0.61.3-{linux-x64,macos-arm64,macos-x64,
  windows-x64}` and `xiom-vscode-0.12.0.vsix`. The extension publish steps were
  SKIPPED by the version-absent gate (0.12.0 already live on both
  marketplaces) -- the gate works as designed. The release job's docs dispatch
  SUCCEEDED this time (the owner's PAT scope fix), so the website received
  `compiler-release` automatically. Note: `gh` was active as
  `Lefteris-Ngonart` (no write access to the org) after the token change; the
  compiler lane switched back to `Lefteris-Notas` and pushed with its token.
- **CI health batch (2026-09-22, from PR #3 triage)**: the stdlib pin PR #3
  surfaced PRE-EXISTING CI breakage (CI never runs on main pushes, so the
  suite hadn't been compiled in CI since R51-era changes). Fixed on main:
  1. `crates/xiom/tests/scripting_tests.rs`: `script_cache_get` takes the opt
     level since R51 -- `--all-targets` had never compiled it.
  2. Encoding: `python tools/ascii_guard.py repair --apply` (13 tracked files:
     BOMs + typographic chars) and pinned `actions/checkout` in `dco.yml`
     (repo policy requires full SHAs; the DCO workflow was unpinned, so every
     DCO run failed with "action is not allowed").
  3. `paths.rs`: the r31 skip-guard test is now CI-aware (`XIOM_REQUIRE_STDLIB=1`
     panics by design) and the crb3c home-candidate assertion is
     host-separator-agnostic.
  4. **JIT on Linux (real bug)**: the shared-lib JIT links the runtime C inline;
     without `-fPIC` ld rejects the runtime TLS relocation
     (`R_X86_64_TPOFF32 ... recompile with -fPIC` ->
     `failed to set dynamic section sizes`). The driver now passes `-fPIC` for
     `-shared` on non-Windows. Linux jit tests 4/4.
  5. e2e stdlib guard accepts BOTH layouts (`xiom/io.xi` flat or
     `xiom/io/io.xi` module dirs); the flat-only probe made every guarded test
     panic under `XIOM_REQUIRE_STDLIB=1` in CI even though the checkout existed.
  6. Test fixture paths normalized `\` -> `/` across five test files (~2400
     literals) for Linux CI portability; intentional escape sequences
     untouched.
  PR #3 outcome: ubuntu-latest fully GREEN on cd42f5d7; windows-latest failed
  only `e2e_p1_contract_methods` (RUN-time access violation `-1073741819`),
  deterministic on the runner (2/2) but NOT reproducible locally with either
  stdlib pin, debug or release compiler, NASM present (RC=0 against the
  v0.61.3 tree). PR closed as superseded and the pin landed directly:
  **`9f78c853` STDLIB_VERSION -> `stdlib-v0.61.3`**.
  Follow-ups: (a) Windows-runner-only AV on `e2e_p1_contract_methods` (CI-side
  toolchain triage); (b) the `xiom-benchmark-chaos\reference\systems\*` e2e
  paths are monorepo remains -- the benchmark moved to its own repo under the
  xiom-project org and now uses the DOWNLOADED compiler; consider dropping or
  relocating those tests; (c) main CI still does not run on pushes (PR +
  dispatch only), so main health is only checked when a PR opens.
- **Registry staging canary VERIFIED + stdlib 0.61.3 (2026-09-22)**: the
  registry lane independently verified `xiom-std@0.61.3` on staging
  (provenance: xiom-lang/stdlib / publish-registry.yml / refs/heads/main /
  runUrl; sha256 + signature re-checked against the served tarball). Stdlib
  release 35726136818: validate + both gates + package + canary-dispatch all
  green; the **pin-PR job failed** on `GraphQL: Resource not accessible by
  personal access token (createPullRequest)` -- the branch was pushed, so the
  compiler lane opened **PR #3** (pins `STDLIB_VERSION` to `stdlib-v0.61.3`).
  Keygen hint corrected (`898ab391`): publisher keys are ephemeral per run;
  consumers pin the REGISTRY key via `xiom pkg trust`. **PAT scope gap**
  (owner): `XIOM_RELEASE_TOKEN` needs Contents: read/write on
  `xiom-lang/website` (docs dispatch 403) and Pull requests: read/write on
  `xiom-lang/xiom` (pin-PR 403). Production debut: approve run 35726136811
  after ops deploys `trusted-publishers.json` (refs/tags/stdlib-v*) and
  rebuilds production; packages canary still needs the curated batch +
  packages staging entries in ops' file.
- **R67 FIXED (2026-09-23, benchmark/option-porter relay)**: `Ok(x)`/`Err(x)`
  (and `Some`/`None`) inside a function returning a USER struct whose name
  contains "Result"/"Option" (`TestResult`, `Options`) built the WRONG struct
  -- the ctor sites chose the container with `ctor_ret.contains("Result")`, so
  `%struct.TestResult` became the Result type (clang: "invalid getelementptr
  indices"; porter workaround was helper ctors). Fixed via the strict
  container-leaf test `is_llvm_container_struct` (bare `Result`/`Option`,
  module-qualified, or concrete `Result__A__B`/`Option__T`) at all four ctor
  sites; concrete `Option[Point]`/`Result[Point, Str]` returns still build
  their concrete containers. Lock `e2e_m120_ctor_user_struct_return` + CI line.
  Full e2e **2368/2368**.
- **R68 FIXED (2026-09-23, packages relay -- legacy nested `extern`)**:
  `extern "C" { ... }` inside a function body (the audio_beep/legacy-package
  idiom) fell into the expression parser and produced the misleading
  `P001: 'extern' is a reserved keyword and cannot be used as an identifier`.
  The parser now parses nested extern blocks in `parse_block` and hoists them
  to the module level immediately before the enclosing declaration (all three
  top-level loops), so the FFI symbols are declared before their call sites;
  duplicate per-function blocks merge. Parser unit tests + probes (main,
  second fn, if-body, block-form module, duplicates) pass; lock
  `e2e_m121_nested_extern` + CI line. The remaining rule-drift inventory
  (declarations without terminators, extern/unsafe contract requirements) is
  a dialect-migration policy item, not a parser bug; the `xiom.ffi` triage
  abort lives in the packages harness (message absent from this repo).
  Full e2e **2369/2369** (R67+R68 batch).
- **R69 FIXED (2026-09-23, the queued generic `T.to_str()` denormal)**:
  root cause was NOT the design question -- `local_xiom_types` is global and
  never cleared, and the monomorphisation param loop recorded only
  Vec/array/fn params. A generic `x: T` kept a STALE `x: Float64` from
  xiom.fmt, so `fn show[T](x: T) -> Str { return x.to_str(); }` lowered
  through the Float64 conversion: show(99) printed the i64 bits as a double
  (4.891e-322), show("hi") printed 0. The mono path now mirrors `compile_fn`
  and records the substituted param type (T=Int -> "Int"; unresolved T stays
  "T" and falls through to the correct erased-LLVM inference). Verified with
  `tmp/cleanbench/to_str_edges.xi` (now 42/1.5/2.5/true/hello/7/65/99/123)
  and the T = Int/Str/Bool/Float64/UInt + two-param lock
  `e2e_m122_generic_param_type` + CI line. Full e2e **2370/2370**.
- **R70 FIXED (2026-09-23, packages relay -- `for x in <collection>`)**: the
  For lowering treated EVERY iterable as `Range{start,end}`: a `%struct.Vec`
  used its DATA POINTER as the index and stored `data+1` back into field 0
  (silent data-pointer corruption), so Vec loops ran ZERO times (heap address
  > len) and any Vec touched by a loop produced BUG-17-family garbage (the
  geometer's "str_len garbage on Vec[Str] elements"); `&Vec` iterables emitted
  invalid IR (clang "invalid getelementptr indices"). The checker bound the
  loop var to Int ("simplified"), so `for s in vec_of_str` mistyped.
  Collections now lower to real element loops (Vec value/`&Vec` header/array
  literal/fixed array; typed element loads; break/continue/nesting intact);
  `range_inclusive`/`0..=b` lower with end+1; real `Range` values keep
  {start,end}; anything else fails LOUDLY. Checker: element-typed loop vars +
  `Vec[elem]` registered for array-literal bindings. Lock
  `e2e_m123_for_in_collections` + CI line. checker 195/195, parser 101/101,
  feature-reg 510/510, integration 129/129, robustness 63/63, fuzz 24/24,
  full e2e **2371/2371**.
  Remaining relayed (package repros needed; local probes pass): indexed
  `Vec[fn]` calls, untyped `Vec[Int]` reads lowered as Str compares, and
  `byte_at(...) == UInt8` for bytes >= 128.
- **R71 FIXED (2026-09-24, queued `all`/`none` stubs)**: the method form
  returned TRUE for every collection in programs without the core module
  (legacy `xiom_all`/`xiom_none` stubs called with len=0) and silently FALSE
  in programs WITH it (`v.all(pred)` resolved to the generic `core.all`,
  never monomorphised -> returning-zero auto-stub). Both now lower INLINE
  over the Vec header with the closure ABI (env[0] = code pointer, i64
  element args, fail-fast, empty = vacuously true); function-NAME predicates
  fail loudly with a pointer to `xiom.core.all/none`. Method-form calls on
  collection receivers now prefer the inline semantics even when a helper is
  registered; the user-function guard still protects the direct form and
  non-collection receivers (Set/Map/iterator methods unchanged -- m39/m48).
  Lock `e2e_m124_contract_all_none` + CI line; checker 195/195; feature-reg
  510/510; robustness 63/63; fuzz 24/24; stdlib-exec 85/85 (+2 ignored);
  full e2e **2372/2372**.
- **m125 FIXED (2026-09-24, stdlib API freeze -- the last red gate)**: the 9
  missing frozen signatures were RENAME-ONLY drift vs the pinned
  stdlib-v0.61.3 tree: `async.Executor.*` -> `AsyncExecutor.*` (same 7
  methods/signatures) and `net.http_get/http_post` returning
  `Result[NetHttpResponse, NetError]` (R44 rename). No API removals, so the
  FROZEN snapshot was regenerated with the new names (header documents the
  decision) and both freeze tests are GREEN (2/2). The gate was absent from
  the   CI path; a `Stdlib API freeze` CI step now runs it. Test-only + CI
  change (no compiler code). Checker 195/195 unaffected.
- **m126 FIXED (2026-09-24, deterministic publish bytes)**: `create_tarball`
  shelled out to `tar`/PowerShell, so published bytes varied run to run and
  never matched a release asset. New in-repo deterministic writer
  (`xiom-pkg/src/tarball.rs`): sorted entries, ustar headers with
  `SOURCE_DATE_EPOCH` mtime (default 0), uid/gid 0, fixed modes 0644/0755,
  gzip MTIME 0/OS 255 with STORED deflate blocks -- no new dependencies
  (supply-chain gate stays audited-only), byte-identical archives for the
  same tree. Symlinks/special files refused loudly. New
  `xiom pkg publish --tarball <PATH>` promotes exactly the given bytes
  (prints their SHA256; temp-only cleanup). 5 new unit tests + Python
  `tarfile` round-trip validation; pkg 72/72 (was 67). Operational note
  unchanged: re-canary after a release re-run.
- **Packages relay triage + Phase 0 (2026-09-24)**: the packages lane sent
  three minimal repros (probes in
  `C:\Users\lefte\AppData\Local\Temp\kilo\fp-probe\`), REPRODUCED on the
  current build (R66-R72 + m127) and confirmed as the fn-value / generic-mono
  ABI family:
  * `fp4_structfield.xi` (`type Op = { f: fn(&Int) -> Int; }`; `(op.f)(&3)`)
    -> compile fails: clang `%tmp16 defined with type ptr but expected i64`
    in `inttoptr i64 %tmp16 to i64 (i64)*` (fn-typed struct-field load not
    coerced for the call path). Concrete, small, highest-priority of the three.
  * `fp5_two_params_scalar_str.xi` (generic `conv[T, U]` with `U = Str`)
    -> compiles, returns the wrong Str (rc=1 vs the green U=Int control).
    The mono'd closure/return ABI uses the erased/generic type for U.
  * `fp4_maptou.xi` (generic `Vec[U] map`, `U = Str`) -> runtime AV
    `-1073741819`; `fp6_diag_vec.xi` (U = Int) -> rc=100 (`w[0]==0`: generic
    `Vec[U]` push/element stride writes the wrong value). Concrete
    `fp4_vecmap.xi`/`fp5_two_params_scalar.xi` controls pass.
  These fold into **Sprint C (fn-value convention unification)** and extend
  it with the generic-mono return/element ABI; probes accepted -- packages
  will commit them under `docs/repro/` (copies kept in `tmp/fp_probe/`).
  Hygiene: `module x;` (trailing semicolon on a brace-less module header)
  -> P001 at the `;`; the diagnostic is clear but the spelling is natural --
  add a small parser tolerance (queue item FE-17).
  Phase 0 executed: local `stdlib/` was 6 commits BEHIND the pin
  (`385e1e4` vs `stdlib-v0.61.3` = `c7b4027`); refreshed to the tag and
  re-gated -- api-freeze 2/2, stdlib-exec 85/85 (+2 ignored) ON THE PIN. The
  full e2e on the pin runs with the next batch (Sprint A), so the recorded
  2373 is pre-pin-honesty only for the suites re-run here.
- **Release strategy REVISED (owner, 2026-09-24 -- Git free-plan build
  minutes)**: **ONE combined release**, no intermediate tags. Ordering:
  1. Compiler lane completes Sprints A (front-end P0), B (P1), C (fn-value +
     the three packages repros), plus the small FE-17; commits/pushes are
     free, so all work lands on `main` first.
  2. Stdlib lane completes its A-list (coverage waves, dedup TUs, tzdata
     phase 2, untested-surface classes, stub re-triage) and cuts ITS next
     release (stdlib repo budget).
  3. Compiler bumps `STDLIB_VERSION` to the new stdlib tag, runs the full
     gate list on the pin, and cuts the single compiler release (recommend
     **v0.62.0**: user-visible behavior changes, doctor v2, publish alias).
  4. Registry re-canary after that release (assets deterministic; same bytes
     on re-run, but new tag = new asset).
  TLS/schannel FFI hardening (stdlib B-list, gates any HTTPS/TLS claim) must
  either ship in this same window or be explicitly excluded in the release
  notes.
- **Website "What's new" compliance (2026-09-24)**: new tool
  `crates/xiom-release-notes` (std-only, no deps) implements the website
  contract (schema v1, `xiom-lang/website` `docs/release-notes-schema.md`):
  `convert` parses `release-notes/<tag>.md`, merges the stdlib fragment at
  `<stdlib>/release-notes/<tag>.md` (highlights default to `kind: stdlib`),
  validates HARD (ASCII/plain-text/no internal ids or hashes, summary 1-240
  without a version, 1-6 highlights, title<=60, text<=320, kind enum, present
  `breaking` where `- None.` becomes `[]`, docs https), and writes
  `release-notes/<tag>.json` deterministically; `verify` regenerates from the
  markdown and fails unless the committed JSON is byte-identical. Wired:
  `release-notes/TEMPLATE.md` + `README.md`, a CI test step
  (`cargo test -p xiom-release-notes`, 6 unit tests), and the release workflow
  gate BEFORE `gh release create` (stdlib checkout at the pin + toolchain +
  `verify`), plus `client_payload[notes_path]` in the `compiler-release`
  dispatch so the website lane can publish the same file to
  `dl.xiom-lang.org/releases/<tag>/release.json` and set `"notes": true` in
  `releases/index.json` (requirement 5 lives in the website/ops lane).
  Smoke-tested end-to-end via the CLI (convert + verify + JSON parse with
  Python; fragment kinds merged correctly).
- **stdlib relay #2 triage (2026-09-24)**: recorded, fixes scheduled:
  * E001 conservatism: deterministic repro at their
    `tools/probes/evidence/p_e001_borrow_conservatism.xi` (pattern: a
    `&local` call, then a later `&mut local` call; 7 warnings in
    `smoke_collect_sparse`, compile/run green). New checker item: E001 must
    not fire for non-overlapping/consumed temporary borrows, WITHOUT weakening
    the genuine-overlap warning. Scheduled with Sprint C.
  * Generic fn-pointer cross-type matrix: 4 repros filed in their
    `tools/known_failures/` (fnptr Int->Str by-ref; map Int->Str and
    Int->Float64 by value; Option[Int].map[U]; sort_by_key[Int,Str] silent
    mis-sort). Matrix: concrete and same-type generic callbacks are CORRECT;
    CROSS-TYPE callback returns are wrong (Str->Int is correct). Stdlib
    surfaces to avoid until fixed: `sort_by_key[Int,Str]` and
    array/Range/MapIter/Option/Result `.map` with cross-type U; they locked
    the working surface with `smoke_sort_by_key`. Same root family as the
    packages' fp probes: the call/return ABI must follow the MONOMORPHISED
    instantiation, not the erased generic signature.
  * m125 closed independently on compiler main (freeze 2/2 in 46s).
  * Shape re-probes confirmed R67/R68/R69/R70/R72 and the still-open
    `Vec[fn]` literal / `.new()+push` AVs (matches our m127 residuals);
    their green lock `p_r70_pending_shapes.xi` is parked for promotion in the
    pin-bump commit.
- **Front-end audit + backlog (2026-09-24, NOT implemented)**: after a Win11
  first-run report (`xiom doctor` said LLVM missing although LLVM was
  installed; it pointed at the dead `xiom install llvm`), a read-only audit of
  everything a new user sees is in **docs/FRONTEND_AUDIT.md** with a
  prioritized backlog (FE-1..FE-16) and two decision briefs. Headlines:
  doctor's LLVM/NASM probes are PATH-only while the driver falls back to the
  standard locations (FE-1/FE-3, root cause of the report); doctor's
  remediation string is a dead command (FE-2); `xiom publish` still runs the
  legacy git-tag flow and `xiom update`'s text points at the wrong thing
  (FE-8); the registry hosts the stdlib as `xiom-std` while manifests depend
  on `xiom.std` and the pkg help examples say `xiom.stdlib` (FE-9, registry
  lane); installers ship 6 of 9 tools and a non-archive runtime layout
  (FE-10); `install_deps.ps1`'s LLVM fallback is an unsigned 19.1.0 download
  (FE-11). D-1 recommendation: keep the stdlib bundled (platform dep) and
  align names; D-2: yes, build the agreed `xiom toolchain check|update`
  (POST_RELEASE_PLAN 1), starting with `check --json`.
- **Cross-lane relays (2026-09-24, for awareness -- no action taken)**:
  * stdlib lane: R66-R70 fixes are on compiler `main` but NOT in the v0.61.3
    pin (e2e 2371 there); next pin bump carries them and unlocks the
    `for x in <Vec>` probe shapes. Their
    `stdlib_api_freeze_no_removals` snapshot item is now RESOLVED here
    (m125, rename-only regeneration + CI step) -- the duplication-gate twin
    removal was waiting on it. Their waves remaining (recon ready): math
    number_theory/factorial/combinatorics, text, regex, test, collections
    (~35 safe clauses), error/io tails; dedup translation units
    (hash vs linkedhash, cache vs lru) need the freeze-snapshot regen (done);
    tzdata phase 2 + untested-surface tail (44 struct-param fns without a
    usable ctor, non-scalar fn params, 83 generic fns) needs new
    gen_call_probes.ps1 generator classes.
  * registry lane: recorded in their SESSION.md at 5c1cdcb. Policy now reads
    that deterministic assets make re-runs byte-stable, but re-canary after
    (a) any asset regeneration or (b) any change to release packing (the
    canary validates the auth/mapping/provenance path and the publish-time
    signature, not the bytes). `xiom pkg publish` still re-packs, so
    byte-identical staging->production promotion needs the client to pack
    deterministically or to publish an existing tarball -- m126 delivered
    BOTH (deterministic writer + `publish --tarball`), so the registry lane
    now has the client-side half; they noted it as optional/unscheduled.
    Nothing pending from ops or registry config.
  * benchmark lane: `COMPILER_VERSION` still pins the absent v0.61.0;
    recommend bumping to v0.61.3 and refreshing the STATUS fields after
    re-running the four suites.
- **m128 FIXED (2026-09-24, registry relay -- package/toolchain correlation)**:
  `xiom pkg publish` never sent the registry's per-version `compiler` field
  (the server reads `req.body.compiler`, capped at 64 chars), so the website
  could not correlate packages with toolchain releases. Publish now resolves
  the tag from `--compiler <tag>` (wins) or the manifest's optional
  `compiler: "v0.61.3"` field, validates it as `vX.Y.Z` (suffix allowed,
  1-64 ASCII), prints it, and adds it to the multipart fields; publishing
  without one warns loudly (not fatal). The field starts flowing with the
  next release; package repos should add `compiler:` to their manifests.
  pkg tests 74/74 (2 new).
- **m127 FIXED (2026-09-24, packages relay -- indexed `Vec[fn]` calls)**: an
  array literal of fn REFERENCES stored the RAW code address as the element
  while every call path uses the closure ENV convention, so `fns[i]()` loaded
  field 0 from machine code -> deterministic AV (the relayed
  `xiom.test.run_all`; workaround `run_test_at(index)`). `compile_array_as_vec`
  now wraps fn-reference elements into envs via `wrap_fn_ref_env` (B-007
  thunk), with the return type derived from the `fn(...) -> R` spelling. Lock
  `e2e_m127_fn_vec_indexed_calls` + CI line; checker 195/195; feature-reg
  510/510; integration 130/130; robustness 63/63; fuzz 24/24; full e2e
  **2373/2373**.
  OPEN (documented in COMPILER_BUGS, repros in tmp/probe_p1/):
  `Vec[fn].new()+push` elements, element-to-local calls, and fn-element calls
  through struct fields still break -- they need a unified fn-value
  convention across generic instantiation, element tracking and the raw-code
  call path (dedicated refactor).
- **Item-2 VERIFIED GREEN (2026-09-23)**: `stdlib_tests::
  stdlib_all_modules_compile_to_ir` passes on the current pin (with and
  without `XIOM_REQUIRE_STDLIB=1`); the R62-era ascii85/`float_to_string`
  findings were resolved by the stdlib pin refresh. Remaining red:
  `stdlib_api_freeze_no_removals` (9 stale snapshot signatures
  `async.Executor.*`, `net.http_get/http_post`; red at baseline) -- cross-lane
  snapshot regeneration, not a compiler bug.
- **Cross-lane relay (benchmark repo, 2026-09-23)**: releases v0.61.1/v0.61.3
  exist but the benchmark's `COMPILER_VERSION` still pins v0.61.0 (absent).
  Recommend bumping to v0.61.3, then re-run the four benchmark suites and
  refresh the STATUS compiler fields (they currently record the pin while the
  suites ran on the installed v0.61.3). Publishing stays blocked on repo
  protection + registry scope additions (owner). Not actionable in this repo.
- **R66 FIXED (2026-09-23, P1-4 contract methods -- the Windows-CI AV)**:
  `e2e_p1_contract_methods` AV'd (`-1073741819`) only on `windows-latest`;
  locally it returned a WRONG answer instead (sorted `[1..5]` reported false).
  The contract lowering passed a pointer to the receiver VALUE to
  `xiom_is_sorted(i8*)`/`xiom_contains(i8*, i64)`; the runtime reads `data[0]`
  as the element COUNT, and for a `%struct.Vec` that word is the DATA POINTER
  -> unbounded scan (runner: unmapped memory; locally: garbage words decided
  the answer). `is_sorted`/`contains` now lower INLINE over the real Vec
  header (len/data/elem-size; sign-correct int loads, `fcmp` for floats,
  `strcmp` for Str; array-literal/fixed-array receivers bridge to a heap Vec;
  unsupported element kinds and non-collection receivers fail LOUDLY).
  Lock `e2e_m119_contract_method_values` + CI line; the p1 fixture now asserts
  values; integration tests updated (inline scan + loud-rejection negative).
  `all`/`none` keep the legacy `len=0` stub (not the reported AV).
  Full e2e **2367/2367**; feature-reg 510/510, stdlib-exec 85/85 (+2 ignored),
  diff 24/24, robustness 63/63, fuzz 24/24. NOTE: `stdlib_api_freeze_no_removals`
  is RED at 9 missing signatures (`async.Executor.*`, `net.http_get/http_post`)
  and fails IDENTICALLY at the pre-R66 baseline -- snapshot vs pinned-tree
  drift, cross-lane (stdlib lane), NOT this change.
- **R65 FIXED (2026-09-22, stdlib p_platform_env)**: `xiom.env.OS/ARCH/FAMILY`
  were hardcoded literals in the stdlib ("windows"/"x86_64"), so a Linux build
  reported windows while runtime detection said linux. Codegen now computes
  them from the TARGET at the module-qualified reference site
  (`target_platform_constants`: wasm -> unknown/wasm32/wasm; aarch64/riscv64
  -> linux/<arch>/unix; Native -> the compiler's own OS/ARCH/FAMILY), and
  seeds the leaf key for bare references. Verified on Windows and Linux (WSL)
  with the stdlib's exact probe: `env.OS=[linux] env.FAMILY=[unix]
  env.ARCH=[x86_64]`. Lock `e2e_m118_env_platform_constants`; full e2e
  2366/2366. Cleanup note: remove `tmp/` archive extracts after verification
  (duplicate stdlib trees trip `e2e_m17_zero_warnings`).
- **R64 (2026-09-22, AI context + post-release spec)**: `AI_CONTEXT.md` at the
  repo root is now the single source of truth for toolchain facts: compiled
  into `xiom-mcp` via `include_str!` and served as
  `xiom_workflow_guide {topic:"context"}` (verified over stdio), shipped in
  every archive as `lib/AI_CONTEXT.md` (release staging + `package.ps1`/sh),
  and renderable by the website docs. Refreshed `crates/xiom-mcp/src/guides.rs`
  where stale: registry URL is now `registry.xiom-lang.org` (was `.com`),
  `xiom pkg` command set (search/info/install/publish/keygen/sign/lock/list),
  OIDC publishing note, `xiom run --opt-level/-O` + script-cache temp
  fallback, `--timeout`, the bundled `z3`/`xiom-wasm.wasm` in the binary
  table, and the long-stale "716 tests" note. Added
  `docs/POST_RELEASE_PLAN.md` spec'ing the verified toolchain updater and the
  MCP `get_contracts`/`search_symbols` tools (owner-approved, post-0.61.1).
- **CRB-8 / RELEASE v0.61.1 PUBLISHED (2026-09-22)**: first public release.
  Two live issues found and fixed during the tag run:
  (a) the VSIX publish to the VS Code Marketplace was rejected by the
  name-similarity policy (`Similar extension display names: ... Language
  Support`); `displayName` is now **"XIOM Toolchain"** (package id stays
  `xiom-lang.xiom`, version 0.12.0).
  (b) The `v0.61.0` tag could not be moved to the fix commit -- the repo's
  release-tags ruleset blocks tag deletion/updates (verified: remote rejected)
  -- so the release was cut as **v0.61.1** (workspace version + README bumped;
  all nine tools report v0.61.1). The `v0.61.0` tag remains as a dead,
  release-less tag; nothing was ever published under it.
  (c) The GitHub Release job failed only at the LAST step, the
  `compiler-release` dispatch to xiom-lang/website: `Resource not accessible
  by personal access token (403)` -- `XIOM_RELEASE_TOKEN` lacks Contents:
  read/write on the website repo. The step is now `if ! gh api ...` +
  warn/exit 0 so a docs-dispatch failure can never fail a release; the PAT
  still needs extending for automatic docs publishing.
  Verified live: GitHub Release `v0.61.1` carries `SHA256SUMS`,
  `xiom-0.61.1-{linux-x64,macos-arm64,macos-x64,windows-x64}` and
  `xiom-vscode-0.12.0.vsix`; the linux archive contains `bin/xiom-pkg`,
  `bin/xiom-dbg`, `bin/z3` and the `xiom-std` manifest; VS Marketplace
  `xiom-lang.xiom` 0.12.0 and its item page return 200; Open VSX 0.12.0
  returns 200. Staging canary prerequisites are satisfied.
- **CRB-7 (macOS release enablement, 2026-09-22)**: enabling
  `RELEASE_BUILD_MACOS` exposed two real portability bugs, both fixed:
  (a) `.cargo/config.toml` carried target-wide `-Wl,-stack_size` rustflags for
  macOS, which Apple's `ld` rejects for non-executables -- every proc-macro
  dylib failed (`ld: -stack_size option can only be used when linking a main
  executable`); the macOS entries are removed (macOS main-thread stack is
  8 MB; use `cargo:rustc-link-arg-bins` if a bin ever needs more).
  (b) `std::arch::is_x86_feature_detected!("avx512f")` sat under a
  `cfg!(target_arch = "x86_64")` RUNTIME check, so it still COMPILED (and
  failed) on arm64; the block is now `#[cfg(target_arch = "x86_64")]`.
  (c) `macos-x64` moved from the retiring `macos-13` (queued >1 h) to
  `macos-15-intel`; the release job now `needs: [build, vscode, build-macos]`
  with `!failure() && !cancelled()` so the optional macOS dependency neither
  skips nor stalls a release. Verified: dispatch 35679231885 is green for
  guard + linux-x64 + windows-x64 + macos-arm64 + macos-x64 + VSIX; the
  downloaded `xiom-0.61.0-macos-arm64.tar.gz` contains all nine tools
  (incl. `xiom-dbg`), `z3` + `libz3.dylib`, `LICENSE-Z3`, and the correct
  `xiom-std` manifest.
- **R63 FIXED (2026-09-22, playground C3/C6 pack)**: script-run `--opt-level`
  dispatch behind leading global flags + `-O<n>` spelling; JIT/script cache
  falls back to `$TMPDIR/xiom_jit` when HOME is set but unwritable;
  `STDLIB_VERSION` bumped from `stdlib-v0.60.0` to stdlib main
  `385e1e44fac37a9403cd04cdb5e13d4c122a8710` (correct `xiom-std` manifest,
  env shim, waves 15-20), local checkout refreshed. Gates on the new pin:
  e2e 2365/2365, stdlib-exec 85/85, feature-reg 510/510, diff 24/24.
- **R62 FIXED (2026-09-22, playground request #2)**: fmt reachable-only peek
  -- primitives peek nothing (codegen builtins), floats peek the small
  `xiom.convert`, generics keep the R47 fmt fallback; `peeked_leaves` keeps
  `float_to_string` through the injection reachability filter and the codegen
  float builtin resolves its symbol via `fn_symbol_map`. Bench:
  to_str/hello emit-ir ratio 1.81x -> 1.12x, loop_200 -31%; output `42`/`1.5`.
- **R61 FIXED (2026-09-21)**: R7 residual + interface-ABI ruling. (a) local
  explicit-generic calls (`var h = make_holder[JsonValue]()`) never recorded
  the substituted return type, so `h.values[0]` lost the element type and the
  index read loaded the first 8 bytes of an inline aggregate as a pointer
  (0xC0000005 in `p_generic_push`/`p_gp_b`/`p_gp_c`); `infer_call_return_xiom`
  now substitutes `Expr::GenericCall` explicit type args. (b) `impl
  Trait[Args]` self PARAMs are retyped from `Self` to the impl type (T001).
  (c) zero-generic entries in `generic_fn_decls` no longer route into the
  generic path (silent 0/no call). (d) RULING: the interface value-receiver
  ABI is unimplemented; codegen now rejects aggregate args for interface-typed
  params LOUDLY (`p_hash_probe` no longer silently returns 5381). Locks
  `e2e_m116_generic_ctor_field_index`,
  `e2e_m117_interface_value_abi_rejected`; full e2e 2365/2365.
- **R59/R60 FIXED (2026-09-21, stdlib relay findings)**: (R59)
  `p_result_tuple_vec_loop` -- the enclosing match arm's result slot leaked
  into nested loop bodies (`oid.push(v)` as a while body's last statement
  stored `%struct.Vec` into the arm's `%struct.Result` slot); While/For/Spawn
  bodies now save+clear `match_result_ptr`. (R60) `p_ref_tuple_mangle` --
  reference-typed tuple elements kept their `&` ("&Vec" -> mangled
  `Tuple__&Vec__Vec`) and container-ctor elements were named by their erased
  LLVM type (`Vec[UInt8].new()` -> "Int"); the expr-side namer strips
  reference markers and BOTH namers resolve container-ctor elements to the
  container base (non-ctor calls keep their qualified inferred name).
  Locks `e2e_m114_result_tuple_vec_loop` / `e2e_m115_ref_tuple_mangle`; the
  three locks an over-broad shared-namer version broke (m37/m44/m48) are
  green. Full e2e 2363/2363.
- **R58 / L8-14 FIXED (2026-09-21)**: Map[Str,Str] morse trap. Payload-type
  resolution for `unwrap`/`unwrap_or` only handled Ident receivers, and a
  `&Map[Str, Str]` param's tracked type was truncated to `"&Map"` (args
  dropped), so `morse.get(ch).unwrap_or("?")` returned the Str payload as a
  raw i64 handle and `result + code` printed pointers. Fixed
  `ref_preserving_name` (bracket-preserving via `type_annotation_name`),
  `generic_container_last_arg` (&/&mut stripping), and both unwrap builtins
  (call receivers resolve via `scrutinee_payload_xiom`; aggregates unbox via
  `try_unbox_payload`). L8-14 prints all four lines; lock
  `e2e_m113_map_str_str_morse`; full e2e 2361/2361.
- **R57 / L3-50 FIXED (2026-09-21)**: `?` on a Result with a tuple payload.
  `let (a, b) = two()?` bound both names to the raw boxed-tuple handle
  (`a + b` printed pointer arithmetic). The `?` handler now unboxes AGGREGATE
  payloads out of the erased i64 slot via `try_unbox_payload`: tuples
  (`(Int, Int)` -> `Tuple__Int__Int`), nested containers, and registered
  structs/enums; primitives/Str/floats stay raw i64; Err propagation is
  unchanged. L3-50 prints 8 deterministically. Lock
  `e2e_m112_try_result_tuple`; full e2e 2360/2360.
- **R56 / L5-40 FIXED (2026-09-21)**: `Map[Int, Vec[Str]]` container payload.
  Decision: container elements are INLINE (32-byte `%struct.Vec` header in the
  slot, matching `Vec[Vec[T]]`); concrete Option/Result layouts keep inline
  payloads. Coordinated five-site change: (1) both explicit type-arg fallbacks
  render nested args via `type_arg_to_name` + mono substitution
  (`Map.new` was `_Int_Int`); (2) `infer_generic_ident_type` prefers the
  tracked local type with args over the erased LLVM slot (`Map.insert` was
  `_Int_Vec`); (3) `record_field_vec_elem` accepts container element types
  (index read now memcpys the inline header); (4) the mono signature builder
  lowers substituted container names (`V` -> `Vec[Str]`) to `%struct.Vec`
  instead of i64; (5) match payload binding resolves fields through the
  CONCRETE `%struct.Option__*`/`Result__*` registration when present. L5-40
  prints 2; lock `e2e_m111_map_vec_container_payload`; the two locks earlier
  attempts broke (`e2e_fnptr_vec_index_call`, `e2e_m71_concat_index_elem`)
  are green -- fixed-array brackets are excluded from container detection.
  Full e2e 2359/2359.
- **R55 / L6-40 FIXED (2026-09-21)**: module-scoped generic factory whose T
  is fixed only by a LATER call (`var runner = plugin_runner.create();
  plugin_runner.add_plugin(&mut runner, EchoPlugin{});`) mono'd `run_all` with
  T=Int -> C001. Added a bounded function-body evidence pre-pass
  (`prepass_generic_type_evidence`, decl.rs) that runs before each body is
  emitted, records the factory call site's concrete types (callee byte-span
  key, consumed before the `0` fallback) and seeds the binding's container
  type; `infer_call_return_xiom` resolves module-qualified generic returns
  from the recorded instantiation or the pre-pass evidence (the `var` arm
  records the binding before compiling its initializer). Lock
  `e2e_m110_generic_factory_evidence` (fixture
  `tests/regression/m110_generic_factory_evidence`); full e2e 2358/2358.
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
  Tests: pkg 72/72, mcp 39/39. Live smoke: search/info JSON + `xiom pkg`
  dispatch verified against the staging registry.
- **R53 staging verification (2026-09-21, registry relay #2)**: re-verified
  live against `https://staging.registry.xiom-lang.org` via the `XIOM_REGISTRY`
  override. MCP `search_packages({category:"core"})` and
  `search_packages({query:"matrix"})` both return `xiom.math` with
  categories/keywords populated; the `matrix` hit is keyword-only (the name is
  `xiom.math` and the description has no `matrix`), proving keyword indexing
  is consumed. MCP `package_info("xiom.math")` returns description, categories,
  keywords, license, repository, latest, and per-version
  sha256/signature/publicKey/dependencies/yanked. CLI parity holds for
  `xiom-pkg` and the `xiom pkg` forwarder (`search --category core --json`,
  `search matrix --json`, `info xiom.math --json`). Wire-shape gaps reported
  to the registry: index top-level `registry`/`version` are declared but
  unused and `updated_at` is ignored (no schema-version negotiation);
  version-level `size`, `published`, `yankedAt`, `yankReason` and the
  duplicated per-version metadata block are dropped; the server `/search`
  endpoint emits `results` while the client `--json` envelope uses `packages`
  (the client filters `/index.json` locally, so a server-side search switch
  would need a mapping); `/categories` has no client surface.
- **R52 payload/binding batch (2026-09-19, playground audit S19)**: closed
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
- **R51 L4 cluster (2026-09-19, playground audit S19)**: closed the four
  remaining `L4-*` clang failures -- `@pre` receivers in `.len()`
  (`items@pre.len()`, `self@pre.items.len()`) now resolve through the
  Vec/field type helpers, and `for i in range(0,n)` resolves the range
  builtin structurally so a user `fn range(...)` cannot hijack the loop.
  The playground's C17 residue is now L6-40 (needs the fixpoint inference
  pre-pass) and L5-40 (container-ABI design, see the L5-40 entry). Locks
  `e2e_m101..m103`.
- **R51 playground requests (2026-09-19, audit S19)**: `--opt-level N` is now
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
Continue the XIOM compiler-lane campaign in E:\xiom-lang\xiom (the compiler
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

