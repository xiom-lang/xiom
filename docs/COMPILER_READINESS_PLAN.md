<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Compiler Readiness Plan (pre-selfhost, production-grade bar)

**Created:** 2026-08-24 - **Branch:** `feat/architect` - **Baseline HEAD:** `81ed009a` (round-15)
**Scope:** compiler crates ONLY (`xiom-lexer`, `-ast`, `-parser`, `-check`, `-graph`,
`-ctfe`, `-verify`, `-codegen`, `-jit`, driver `xiom`, tooling: fmt/lsp/dbg/pkg/ffigen/
doc/mcp/display/wasm). The stdlib audit (`docs/xiom-stdlib-audit-v0.58.md`) and
`stdlib_session.md` belong to the PARALLEL STDLIB SESSION. This session never edits
`stdlib/` files; stdlib-sweep-discovered bugs stay here as COMPILER work.
**Goal:** every finding in `docs/xiom-compiler-audit-v0.58.md` fixed at production
grade, plus the remaining round-15 campaign queue, before the self-hosting push.
Repo goes public AFTER readiness; registry/package testing comes after the split.

Sources merged into this plan:
- Compiler audit top-20 defect list + Phase 0-5 roadmap (`xiom-compiler-audit-v0.58.md`)
- SESSION.md round-15 remaining queue (items 1-8) + old Items A/B
- Operational lessons (probe discipline, baseline verification, suite order)

---

## Stage 0 -- Ground truth (first working block)

| # | Task | Source / notes |
|---|------|----------------|
| 0.1 | Build round-15 binary; quick suites green (checker/parser/ctfe); full e2e in background | laptop = fresh environment; e2e_m16_scripting_exit_zero is environmental (ignore) |
| 0.2 | Resolve geom contradiction: SESSION.md says probe_skew3/4 green at baseline; stdlib_session.md (later same day) says garbage `-4.0` bits. Recreate probes from the documented shape (mono'd fn bodies reading `m[i][j]` through nested `&Vec[Vec[Float64]]` params) and settle it | SESSION vs stdlib_session |
| 0.3 | Consolidate COMPILER_BUGS.md master queue: campaign items + audit findings #1-20 cross-referenced to stages below | both audits |
| 0.4 | Item B: api_freeze/exec harness path sync (STDLIB_MANIFEST/SMOKES references) | old queue; blocks honest verification |

## Stage 1 -- Silent-corruption criticals (audit #1 #2 #3 #7 #8 #14)

- **CTFE rewrite**: explicit-stack iterative evaluator (kills documented stack-overflow
  ICE at xiom-ctfe/lib.rs:842-845), real aggregate constants (no more `Int(0)` sentinel,
  :661-671), `for` bodies evaluated per iteration (:395-402), DEFINED overflow policy,
  session-wide fuel budget, arena-cap panic -> diagnostic.
- **Verifier honesty**: sort-consistent SMT encoding (all-BV or all-Int per function),
  real struct-field encodings (no undeclared uninterpreted fns), type invariants emitted
  (not TODO comments), `unknown` is a DISTINCT verdict (never emit literal `false`,
  :557-559), bounded z3 sessions with concurrent drain + parent-side kill (fixes stdin
  pipe deadlock :672-678).
- **Quick checker/parser wins**: arity check on module-prefix calls (extra args are
  silently dropped today); warnings no longer dropped when the error list is empty
  (check lib.rs:1108-1110); `MAX_EXPR_DEPTH` 24 -> 128 with a structured error;
  debug_assert on the parser's Eof-stream invariant (`advance()` tokens[pos-1]).

Gate: ctfe suite stays green + NEW regression tests (struct/enum consts, recursive
const errors cleanly, for-loop const eval).

## Stage 2 -- Structural foundations (audit Phase 0; findings #6 #15)

1. Byte-offset spans `(file_id, start, end)` threaded lexer->parser->AST; line:col
   derived lazily for Display. Unblocks diagnostics ranges, LSP UTF-16 math,
   incremental, DWARF later.
2. Symbol interning + structural TypeId; DELETE string-encoded type equality
   (`Option == Option[T]`, `_ == Int`, alphabetical wildcard method capture); unify the
   two `types_compatible` copies (lib.rs vs compat/mod.rs). Expect array_zip T001
   (checker tuple-element store on `[N](T,U)`) to flip green here.
   [array_zip T001 flipped green round 36; the orphaned compat/mod.rs copy was
   deleted round 18; round 39 STARTED the structural core: parser + real
   TypeArena interning + `types_compatible` on parsed shapes.
   Follow-on slices: `CheckedType::Named(TypeId)` storage, get_type bare-name
   fallback through the arena, codegen keys on the same canonical form.]
3. Move `expand_impl_blocks` out of xiom-ast into a lowering pass (no Span(0,0)
   injection, no whole-program clones); wire ErrorGuaranteed construction into
   parser/checker for real.
4. Lexer: preserve comments/trivia (unblocks fmt comments + docgen ///); numeric
   overflow -> error token (never silent unwrap_or(0)).

## Stage 3 -- Soundness gates on (audit Phase 1)

- Generic bounds enforced in the FRONTEND (stop deferring to codegen, lib.rs:8079-8102).
- Container args compared structurally; wildcard receivers resolved by inference only;
  coherence-lite impl-overlap checks; exhaustiveness for ARBITRARY enums (not just
  Option/Result/Bool).
- Item A: catalog body type-checking Phase 1 (reachable injected fns) -- user-approved.
- Borrow check stays opt-in outside strict profiles; promoted default documented.

## Stage 4 -- Crash families & codegen truth (campaign + audit #17, sec 8.2)

- Root-cause the CRT-layout family AND the "-O0/-O1 miscompile" TOGETHER (both point at
  UB in emitted IR exposed by layout sensitivity; SIMD flags / 16 clang-variant matrix).
  VERIFICATION DISCIPLINE: baseline rebuild comparison before calling any change a
  regression (git stash + rebuild + rerun + pop).
- json heap layer (0xC0000374: json_nested/json_parse_valid/nested); stack-cookie fns
  (argon2/pbkdf2/io_bufreader/math_edge); clang variants (ptr_offset/io_copy x3/
  hash_values/convert_escape/regex_captures x4); geom family per Stage 0 verdict;
  smoke_convert_narrow_roundtrip triage.
- JIT: implement state migration or honestly descope the claim; guard DLL-unload
  liveness (dangling symbol pointers on hot reload).
- LET-array representation conflict: written up as a JOINT decision doc FOR the stdlib
  session (design call shared with them; M33 let->Vec vs &[N]T array-module fns).
  [DONE round 39: docs/LET_ARRAY_DECISION.md -- let arrays are FIXED arrays;
  P1 annotated-let IR DONE round 40; P2 user-fn &[N]T element-pointer ABI DONE
  round 46; P3 M33 let->Vec deletion + call-site Vec materialization DONE round
  47 (`let a = [...]` binds `[N]T`; passing it to a `&Slice[T]`/`Vec[T]`
  (%struct.Vec) param materializes a heap-backed Vec with the array's elements,
  so push-taking by-value Vec consumers keep working).]

## Stage 5 -- Toolchain trust & security (audit #4 #5 #9-13 #18-20 + hygiene)

- Driver: clap-based arg parsing (drops `wasm|arm|riscv` filenames today), honor
  `xiom.toml [compiler]`, fix sandbox exit-0 false-green (main.rs:763-765), remove
  process::exit from library paths (lib.rs:1232; watchdog thread main.rs:535-542 ->
  cancellation token), randomized temp names.
  [DONE round 41: watchdog -> cooperative cancellation token in xiom-codegen
  (clang child killed on cancel, main thread reports); xiom.toml [compiler]
  timeout-secs honored (CLI wins). Round 45: target-named source files
  (wasm/arm/riscv) are no longer dropped. clap parsing / sandbox
  false-green / randomized temp names remain.]
- Supply chain (HARD PREREQUISITE for the post-split registry phase): client-side
  sha256 verification of every artifact, ed25519 signatures + trust model, lockfile v2
  pinning {name, version, integrity, source} transitively, ureq-only HTTP with TLS +
  timeouts -- DELETE the PowerShell interpolation sites (command injection via
  --registry/XIOM_REGISTRY) and raw-TCP/curl fallbacks; git installs pinned to commit
  hashes; authenticated publish by default.
  [DONE: sha256 verification (audit #4), ureq-only transport + HTTPS enforcement
  (audit #5), lockfile v2 (`xiom pkg lock` writes {version, source, integrity};
  `install` ENFORCES the locked digest when xiom.lock is found, XIOM_PKG_LOCKED=0
  bypasses / =1 requires it). Round 72 also fixed two real manifest-parser bugs the
  lock exposed: deps were never parsed (the map was only cleared) and
  strip_outer_block stripped at the first brace ANYWHERE, wiping unbraced manifests.
  Round 75: ed25519 SIGNATURES + trust model (`xiom pkg keygen` /
  `trust --registry URL --key HEX` / `sign` / `verify`; `install` FAILS CLOSED for
  trusted registries whose artifacts are unsigned or mis-signed, and hints TOFU for
  untrusted ones), ureq-only multipart publish (the last `curl` shell-out is gone,
  `XIOM_REGISTRY_TOKEN` -> Authorization: Bearer with a warning when absent), and
  git dependencies must pin a full 40/64-hex COMMIT (branches/tags refused unless
   XIOM_PKG_ALLOW_MUTABLE_GIT=1). REMAINING (was): transitive dependency closure
   via registry metadata and server-side publish authentication.
   DONE 2026-09-17: server-side auth is registry-side (token scopes/trusted/
   firstParty + actionable 401/403/409/422 codes); the client resolves and
   installs the TRANSITIVE CLOSURE -- `select_version` matches
   `>=,<,<=,>,=,^,~` specs over non-yanked versions (exact pins still resolve
   yanked releases), install walks a cycle-safe deterministic
   `install_closure` whose dependency source is the VERIFIED tarball's own
   `package.xi` (index metadata fallback), and every artifact goes through
   the full sha256 + signature + lockfile path; `lock` pins the closure with
   digests (`Lockfile::from_resolved`). Registry e2e 20/20 (dependency
   fixture + closure install + transitive lock assertions).]
- LSP: integer severities, UTF-16 positions via span table (fixes multibyte panics),
  bounded Content-Length buffers (64 MiB cap), mutex-poison recovery instead of 15x
  expect, incremental reparsing, cross-file index.
  [Partial round 43: integer severities + 64 MiB cap (round 16d); UTF-16
  positions via position.rs and mutex-poison recovery DONE; incremental
  reparsing + cross-file index remain.]
- fmt: defer support (todo!() crash today), comment/shebang preservation (needs Stage 2
  trivia), string-literal escaping on re-emit.
  [Partial round 42: shebang + leading comment/header blocks preserved and
  string/char literals escaped on re-emit (`xiom_fmt::format_source_text`);
  body-inline comment trivia attachment remains.]
- dbg: MI command quoting (injection via evaluate/breakpoints), async MI reader, 
  .xi DWARF mapping (enabled by Stage 2 spans).
  [Async MI reader DONE round 69: dedicated reader thread classifies result
  (`^...`) vs async (`*stopped`/`*running`) records into bounded-wait queues;
  send_mi and poll_stopped can no longer block the DAP forever (non-stopping
  continue reports "running"). `.xi` DWARF mapping DONE round 71:
  `DISubroutineType` node added (the old `type: !{}` made LLVM warn
  "ignoring invalid debug info" and drop ALL DWARF), per-statement
  `!DILocation` attachments emitted for every instruction while `-g` is on
  (buffered nodes flushed at module end; default builds stay metadata-free).]
- Shared JSON diagnostics v1 schema consumed by LSP/MCP/CI.
  [DONE round 45b: docs/JSON_DIAGNOSTICS_V1.md + serde envelope
  (`xiom::diagnostics_json`); all ad-hoc diagnostic printers replaced.]
- Engineering hygiene: cargo-deny/vet, MSRV declaration, workspace version policy
  (four schemes coexist today), cargo-fuzz targets over lexer/parser/CTFE replacing toy
  LCG harnesses, ASAN/UBSAN runs of the differential suites in CI.
  [Round 44 DONE: workspace version/edition/MSRV inheritance (1.86) + deny.toml
  + CI hygiene job (MSRV check --workspace --all-targets + cargo deny); the
  job also caught and fixed a rotted xiom-mcp member (E0063). REMAINING:
  cargo-vet audits, cargo-fuzz targets, ASAN/UBSAN CI runs.]

## Stage 6 -- Performance program (audit sec 8.1)

Real incremental engine (content/signature fingerprints replacing module_path+dep-names;
populate the designed-but-unused L1 parse/L2 check cache tiers); parallel
monomorphisation behind rayon; linker strategy decision (embed lld vs keep the clang
pool); benchmark suite with CI regression budgets (self-build wall time, IR bytes/fn,
hello+matrix workload, cold/warm run latency).

## Stage 7 -- Selfhost gate (then monorepo split -> registry, standing sequence)

- [ ] Self-build compiles with zero ICEs; fuzz corpus (>=1M execs) crash-free
- [ ] Differential test: same binary behavior under -O0/-O2/-O3 and clang/gcc
- [ ] Checker enforces bounds/coherence; borrow check on by default in strict profiles
- [ ] All Stage-5 supply-chain controls active for obtaining the toolchain itself
- [ ] Stage-1 vs Rust-bootstrap output equivalence harness (IR diff or behavioral diff)
- [ ] Full e2e + quick suites green on the release binary

THEN: docs/REPO_SPLIT.md split, then registry + package testing (repo public by then).

---

## Standing defaults (unless overridden by the user)

- Integer overflow semantics: TRAP in debug/profile builds (llvm.sadd.with.overflow +
  branch), WRAP in release. Applied consistently to CTFE/runtime/verifier.
- Verifier reports UNKNOWN as its own verdict; never silently `false`.
- Borrow check remains opt-in until strict profiles exist; then default-on there.
- CRT-layout changes verified against a BASELINE binary rebuild before being judged.
- Probe discipline: `xiom --run -o out.exe file.xi`; read the PRINTED exit-code line,
  never $LASTEXITCODE. Never run two suites concurrently. ascii_guard: staged files
  must be pure ASCII (`python tools/ascii_guard.py repair --apply` if a commit aborts).
- Commit style: conventional messages (fix(codegen)/fix(check)/docs/...), one round =
  one batch + regression tests + doc updates.
