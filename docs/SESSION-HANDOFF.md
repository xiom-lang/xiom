<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-13 (compiler lane; v0.64.4 SHIPPED; v0.64.5 batches m276-m284 landed)

This file captures state for continuing the compiler lane in a fresh
session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Status: v0.64.4 RELEASED; stack caught up; v0.64.5 plan in section 1c

v0.64.4 shipped 2026-10-12T13:37Z (tag on `df493530`; release run
`38256358845` GREEN; assets live). The commit map that produced it
(m254-m275; all pushed) is kept below for reference:

*** SESSION UPDATE 2026-10-27T00:40Z (compiler lane; v0.65.2 in progress) -- READ FIRST ***
- **v0.65.1 remains PRODUCTION.** v0.65.2 is in progress; no tag.
- **P0 arm64 FIXED and merged** (`3e775908a`, merge `1d4de2a1a`): the
  contract evaluator's bare `result is Ok` rebind now loads the Result
  payload field's actual LLVM type (`Result[Str, E]` field 1 is `i8*` on
  arm64) and normalizes to the erased i64 handle. New arm-IR lock
  `e2e_m3752_arm_result_str_contract_payload` runs in the arm64 floor job.
  Full e2e **2648/2648 (4 ignored)** debug+release; CI **40411862697
  SUCCESS** including **arm64 floor (ubuntu-22.04-arm, clang 14)**. PULSE's
  fleet arm64 archive build is the remaining exit.
- **t1 30 MB diagnosed and relayed:** the Vec ABI floors every slot at 8
  bytes (`with_len` zero-fills `n*8`; push/reserve/insert grow `new_cap*8`;
  compiler `vec_slot_size = max(elem_size, 8)`). `Vec[UInt8].with_len(32M)`
  peaks ~282 MB private; the t1 4096/32 pool is not the cause. Stdlib item
  `REL-20261026-2142-compiler` (packed slots + coordinated compiler change).
- **Selfhost Phase 6 is running in parallel** in
  `.kilo/worktrees/selfhost-phase-4-codegen`: value-returning `NkExprMatch`,
  parser tail re-wrap (`diff_ast` 83/83), then m1136 + P6 shapes.
- **LSI/safety:** filed `REL-20261026-2143-compiler-2` (benchmark) for the
  t8/t9 probe source; proposed `xiom --sanitize` (poison/quarantine/guard
  pages), safe-subset uninit/type-punning rules, and a recoverable fault
  mode. Current profile LSI 45 (UAF/uninit SILENT_UB; type confusion
  SILENT_FAILURE).
- Full v0.65.2 plan and the compiler relay-queue mapping:
  `docs/NEXT-CANDIDATE-v0.65.2.md`. Next gates: stdlib packed-slot tag +
  compiler follow-up, Str/Map/JSON/HTTP retention, fleet re-verify, then
  owner GO.

*** SESSION UPDATE 2026-10-26T19:45Z (compiler lane; v0.65.1 RELEASED) -- READ FIRST ***
- **v0.65.1 is PRODUCTION.** Tag `v0.65.1` (annotated
  `c4329915f6a55e994ad7ddc05399059a85a0c796`) on `f2c01cf9c`; release run
  **40388025015 SUCCESS** with **11 assets + SHA256SUMS** (file sha256
  `87565042...`; windows-x64 `69de1e27...` verified). Docs dispatch
  (`compiler-release` -> `xiom-lang/website`) success. Registry
  `xiom-std 0.65.1` signed (sha256 `3bb9b850...`). v0.65.0 is superseded;
  v0.64.9 remains withdrawn.
- Gate: CI **40385480182 SUCCESS** on `f2c01cf9c`; all-platform dry-run
  **40387243828 SUCCESS** on the same commit; final-tree e2e
  **2647/2647 (4 ignored) on both debug and release**; nine hard-RSS gates
  green on both; non-e2e battery green; `xiom.fontraster` 0.1.1 conformance
  11/11.
- Shipping fix: `0eef57018` (trusted unsafe FFI pointer export + fresh
  rebind; sound `fn_owns_return_transfer`), with `778e01d57`/`8d4968522`.
  UI soak flat: 1200 frames, 2 GB cap, exit 0, peak private 1044 MB / WS
  181 MB (was 2908 MB/kill).
- Post-release: `xiom.fontraster` 0.1.1 is LIVE (packages lane,
  `xiom-fontraster/v0.1.1`, sha256 `c5be1ac1...`, run 40376049948; relay item
  verified). The engine should verify the v0.65.1 archive; fill the v0.65.1
  release check; close the holds. Known bounded residual: an actually-taken
  retry rebind leaks only the replaced buffer.

*** SESSION UPDATE 2026-10-26T18:30Z (compiler lane; v0.65.1 CANDIDATE, owner GO) -- READ FIRST ***
- **Owner GO received; cutting v0.65.1.** stdlib tag `stdlib-v0.65.1` is
  live (`9dc69b3`); the local stdlib checkout is pinned to the tag.
- Release tree: workspace/Cargo.lock **0.65.1**; `STDLIB_VERSION` =
  `stdlib-v0.65.1`; `release-notes/v0.65.1.md` converted to JSON with the
  pinned stdlib fragment (5 highlights, 0 breaking, 1 docs); new gate doc
  `docs/RELEASE_GATE_v0.65.1.md`.
- Scope: m375 owned-Vec reclamation (`778e01d57`, `8d4968522`, `0eef57018`),
  the flat 1200-frame `UI_SHOWCASE_ANIMATE=1` soak, `xiom.fontraster` 0.1.1
  (packages `b196c10d`/`cb20373f`; engine `f394645`/`ce8a292`), and the
  stdlib read-helper ownership contracts.
- Evidence so far: UI soak 1200 frames exit 0, peak private 1044 MB / WS
  181 MB (flat; old tree 2908 MB/kill); nine hard-RSS gates green on
  debug+release; non-e2e battery + local CI-equivalent suites green; full
  debug+release e2e 2647/2647 (4 ignored) on the pre-bump tree.
- **Still pending before the tag:** full e2e + battery on the final pinned
  tree; push main; CI green; all-platform dry-run (release workflow_dispatch)
  green; then annotated tag `v0.65.1`, release workflow (11 assets +
  SHA256SUMS), docs dispatch, registry `xiom-std 0.65.1`, lane announcements.
- `xiom.fontraster` 0.1.1 publish remains with the owner/packages lane.
  v0.65.0 remains production until v0.65.1 is live; v0.64.9 withdrawn.

*** SESSION UPDATE 2026-10-26T16:45Z (compiler lane; m375 UI soak FIXED on main, NO release) -- READ FIRST ***
- **The 1200-frame UI soak is flat on the current main tree (not released).**
  Two compiler gaps beyond `8d4968522`: (1) `#[owns_return]` did not exempt
  `as_ptr`/`as_mut_ptr` inside a trusted unsafe block, so `font_shape`'s
  `buf`/`slots` stayed escaping; (2) any bare-local rebind (`buf =
  Vec.with_len(..)` on the retry path) marked the whole local escaping, so
  `buf` leaked on EVERY call, not only the retry. Fix: trusted pointer export
  is non-escaping inside a trusted unsafe block; a fresh-to-fresh rebind keeps
  ownership; `fn_owns_return_transfer` grants trust only for owning return
  shapes (Vec / `Result`/`Option<Vec>` / named aggregate). New gate
  `tests/soak/ffi_owns_return_leak.xi`.
- `xiom.fontraster` 0.1.1 (installed locally in `XIOM_HOME`; publish pending
  owner GO): `#[owns_return]` on `font_shape`/`font_glyph_raster`/
  `font_face_metrics`; engine `xiom.toml` pins `0.1.1`.
- Evidence: UI `UI_SHOWCASE_ANIMATE=1`, 1200 frames, 2 GB cap, exit 0:
  peak private 1044 MB, peak WS 181 MB; sampled private 1037 -> 1011 MB over
  the run (flat; the old tree was killed at 2908 MB private). Nine hard-RSS
  gates green on debug+release; full e2e **2647/2647 (4 ignored) on both
  compilers**; non-e2e battery green (550/130/40/26/78/214/109/43/3; stdlib
  execution 85 passed, 2 ignored).
- Still pending before a next-candidate tag: publish/install fontraster 0.1.1
  (owner GO), stdlib `v0.65.1` tag (stdlib lane, acked, pending owner GO) +
  the `STDLIB_VERSION` pin, compiler candidate bump + notes, CI + all-platform
  dry-run, then owner GO. v0.65.0 remains production; v0.64.9 withdrawn.
- Residual (not release blocking): an actually-taken fresh rebind leaks only
  the replaced buffer (one per retry); the final buffer is transferred to the
  caller. Proper move/refcount semantics for replaced owning headers is future
  work.

*** SESSION UPDATE 2026-10-26T00:20Z (compiler lane; m375/engine Vec ownership FIXED on main, NO release) -- READ FIRST ***
- **m375 / engine Vec ownership is fixed on main `778e01d57`, not released.**
  Root cause: drops were registered only for fresh `let`/`var` Vecs and
  emitted only at function return; call-return Vecs, match-bound
  `Result[Vec]` payloads, aggregate Vec fields, and confined-block locals
  were never registered, and an arena free was a no-op. `Vec.with_len` was
  not recognized as fresh.
- Six hard-RSS gates pass (`tests/soak/run_memory_gates.ps1`):
  `with_len` 5 MB (was 209), fresh-return 5 MB (was 156 kill), `io.read_at`
  5 MB (was 140 kill), editor frame 28 MB (was 550 kill), confined arena
  loop 5 MB (was 1.2 GB kill), break/continue 5 MB. Caps: 128/128/128/512/512/128.
- Non-e2e battery green: feature 550, integration 130, stdlib 40, codegen
  26, xiom 78, checker 214, parser 109, graph 43, API-freeze 3. Full
  debug+release e2e is green on `778e01d57`: **2647/2647 (4 ignored) on
  both compilers**; release build green.
- **Engine cargo report (2026-10-26):** vecprobe mode 0/1 fixed on
  `8d4968522` (5 MB vs 418/390 at 800k); UI link fixed via libc `calloc`
  (no new `xiom_alloc` dependency); `probe_ui2_text`/full `ui_showcase`
  build and run with explicit `XIOM_STDLIB`. The 1200-frame soak still
  hits 2 GB: `xiom.fontraster` `font_shape`/`font_glyph_raster`/
  `font_face_metrics` need `#[owns_return]`, package 0.1.1, and the engine
  `xiom.toml` dependency; relays REL-20261026-1158-compiler and -2. Full
  e2e after `8d4968522` pending. **No cut until the soak is flat.**
- **Next-candidate blockers:** stdlib `v0.65.1` with `#[owns_return]` on
  `read_at`/`read_file_bytes` (relay REL-20261026-0040-compiler-2), engine
  frame builders annotated + the 1200-frame `UI_SHOWCASE_ANIMATE=1` soak
  flat (relay REL-20261026-0040-compiler), then CI + all-platform dry-run +
  owner GO. **No tag.**
- v0.65.0 remains production; v0.64.9 withdrawn. The 2026-10-25 block below
  covers the v0.65.0 release, arm64 printf, `.xiomignore`, and warm-cache
  fixes.

*** SESSION UPDATE 2026-10-25T15:25Z (compiler lane; v0.65.0 RELEASED; next candidate in prep) -- READ FIRST ***
- **v0.65.0 is PRODUCTION** (tag `v0.65.0` / `f527d5a` -> `a0dc452dc`;
  release 40041016248 with 11 assets + SHA256SUMS, all checksums re-verified;
  registry `xiom-std 0.65.0` sha256 `c3629878...`, compiler field `v0.64.8`;
  docs dispatch 40041418782). **v0.64.9 stays WITHDRAWN**; v0.64.8 is
  superseded. Do not pin/deploy v0.64.9.
- **No next tag yet (owner direction 2026-10-25).** Next-candidate fixes on
  main, not released:
  * arm64 `io.print` variadic `@printf` call type (`5b0bedd83`,
    REL-20261024-1101-pulse): canonical libc varargs call types; arm
    `--emit-ir` emits `call i32 (i8*, ...) @printf`; debug AND release e2e
    2646/2646 (4 ignored) before the arm lock; feature lock
    `regress_variadic_libc_call_type_printf` plus e2e
    `e2e_m1101_arm_io_print_variadic_printf` (release P0 list + LLVM 14
    arm64 floor; next-candidate suite 2647 expected). Next-candidate CI
    40206639784: 7/7 green, including the arm64 floor and the release P0
    locks with the new test.
  * `.xiomignore` precedence (`b45f96be9`, REL-20261014-1504-registry):
    ignore suppresses default-refused entries, including under `--force`;
    pkg 97 lib + 1 integration; relay marked fixed.
  * warm script-cache stdio (`cfc17e92c`, REL-20261022-2032-playground):
    cache hit inherits stdin/stdout/stderr and propagates the child status;
    live streaming lock + C25 2/2; relay marked fixed.
  * Stage 6 confinement budget (`2d40764c6`, S7): <=300 ns/transaction,
    <=1.5x hot paths, zero trampoline/guard calls for trusted
    `#[unsafe_direct]`.
- **Remaining still-red rows for the next candidate** (`release-checks/
  compiler-v0.65.0.md`): by-ref fn-pointer trap
  (REL-20261010-1553-playground-2); t3 trampoline ~8x C
  (REL-20261010-1554-benchmark); silent Int+Str coercion
  (REL-20261010-1554-playground-2; warning-vs-error policy decision);
  `.to_str()` W005 stub (REL-20261010-1554-playground-3); RSS retention
  (REL-20261010-1549-pulse-4). The pulse rows blocked on the arm64 build
  (REL-20261010-1549-pulse, -1609) are unblocked and need a re-probe on the
  next archive.
- Next candidate: include the four fixes above, rerun CI + the all-platform
  dry-run, then owner GO before tagging. Cargo:
  `C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin`.

*** SESSION UPDATE 2026-10-23T05:00Z (compiler lane; v0.65.0 P0/P1/P2 GREEN; P3 waiting on stdlib + owner GO) -- READ FIRST ***
- **v0.64.9 stays WITHDRAWN**; production remains **v0.64.8** until the
  v0.65.0 tag. Do not pin/deploy v0.64.9.
- **Main tip `ade98fb68`** (code through the MCU wave). Workspace 0.65.0.
  `STDLIB_VERSION` is still `stdlib-v0.64.9`; change it to
  `stdlib-v0.65.0` only after the stdlib cut.
- **P0 complete:**
  * true 1550 (`dd9256343`): on-demand chained-receiver generic inference;
    `em=[(0,10),(1,12),(2,14)]`, `em[1].0==1`; e2e
    `e2e_m1550_enumerate_tuple_map_true`.
  * 1827/1645/2045 (`45804b195`, narrowed in `44dcf01c0`): direct
    Err(Str)-in-Vec, cross-module Vec-in-tuple, direct Err(Enum.Variant)
    match; three e2e locks.
  * 1529 (`a6ba113ed`): typed fn-pointer operands, no `ptrtoint ptr`;
    x64+arm IR locks; arm64-floor CI job.
  * release parity (`4acd28bd7` + CI): `release-parity` (ubuntu/windows)
    and `arm64-floor`; `XIOM_TEST_COMPILER` harness switch.
  * m1511 (`bf4709a5a`): transitive cycle-safe port.args closure;
    installed fontraster -> harfbuzz build verified.
  * C07 (`44dcf01c0`): arena-guarded Vec Copy-Out; engine
    `probe_rhi_vulkan` PASS 38 checks at 821 MB (v0.64.8: 1391 MB killed;
    v0.64.4: 744 MB); e2e `e2e_c07_unsafe_loop_vec`.
  * battery regressions closed in the same commit: m44/m48, m35_t21,
    parallel-codegen rayon stack.
- **P1:** full battery + full e2e **2646/2646 (4 ignored)** on debug AND
  release; ASAN clean on the 1827/1550 probes; IR/exe reproducibility
  byte-identical. Lane re-verification happens on the v0.65.0 archive.
- **P2 MCU first wave (`2c5a2192d`):** `thumbv6m`, `thumbv7em`, `thumbv8m`,
  `riscv32imc`, `riscv32imac`; `--freestanding`, startup/linker/runtime
  contract (`docs/MCU_FREESTANDING.md`, `runtime/freestanding/`), BYO
  `--linker-script`, QEMU CI `mcu-qemu`. Both hellos link with zero
  undefined symbols. Bindings relay `REL-20261023-0423-compiler`.
- **P3 blockers (only these remain):**
  1. **stdlib v0.65.0** must include the C07 runtime helper
     `xiom_guard_copy_out_if_arena` (`REL-20261023-0242-compiler`) and a
     v0.65.0 notes fragment. Then pin `stdlib-v0.65.0`, regenerate
     `release-notes/v0.65.0.json`, re-run the battery.
  2. All-platform dry-run on the release commit.
  3. Owner GO; then tag, 11 assets, registry `xiom-std 0.65.0`,
     announcements.
- Release notes are authored at `release-notes/v0.65.0.md` (JSON exists
  but must be regenerated after the stdlib fragment). Packaging ships
  `runtime/freestanding` under `lib/runtime/freestanding`.
- Cargo is at `C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin`.
- Continue in `docs/checklists/v0.65.0.md` -> P3 and
  `docs/RELEASE_GATE_v0.65.0.md` -> "P3 blocking dependencies".

*** SESSION UPDATE 2026-10-22T19:52Z (compiler lane; v0.65.0 P0 in progress; v0.64.9 WITHDRAWN) -- READ FIRST ***
- **v0.64.9 is WITHDRAWN** (GitHub pre-release); production remains
  **v0.64.8**. Do not pin or deploy v0.64.9. The stdlib v0.64.9 tag and
  registry exist but are held.
- **v0.65.0 is the active production goal.** Gate:
  `docs/RELEASE_GATE_v0.65.0.md`; checklist:
  `docs/checklists/v0.65.0.md`; paste-ready successor prompt:
  `docs/CONTINUATION-PROMPT-v0.65.0.md`. No candidate/tag until the gate
  is green and owner GO is given.
- Main `dfd234845` plus this handoff commit; workspace/selfhost 0.65.0;
  `STDLIB_VERSION` remains `stdlib-v0.64.9` for development (v0.65.0 will
  pin `stdlib-v0.65.0` after the stdlib cut).
- **Landed since the 18:35Z block:**
  * nested enum-pattern payloads (m1615/1243) at `ecd22a225`: inner
    variant discriminant checks + Float64/Str binding; JSON Number probe
    passes; e2e `e2e_m1615_nested_enum_payloads`.
  * m372/1451 at `ecd22a225`: generic pointee base canonicalization,
    LLVM pointee fallback, pointer-local inference; `nested=42 local=42`,
    e2e green.
  * partial true 1550 at `dfd234845`: `EnumerateIter.map` now binds U from
    `idx_plus`'s tuple return and receiver generics map by receiver param
    position; `collect` still specializes `MapIter.collect_Tuple__Int__Int_Int`
    (U=Int) and `em[1].0` is still 0.
  * battery: feature 549/549, integration 130/130, stdlib 40/40, codegen
    lib 25/25.
- **Open P0 for v0.65.0:**
  1. True 1550: resolve the map call's returned `MapIter[(Int,Int),U]` for
     the `collect` receiver, then add the `em[1].0` e2e lock.
  2. 1645/1827 data-loss cluster: direct `Err(E.Bad(Str))` in Vec (loses
     Str; local-bound control works), `Vec[Int]` inside a cross-module
     `Result[(Nominal, Vec[Int]), Str]` (contents lost), and direct
     `Err(Enum.Variant(..))` match. Probes are in the packages repo; add
     them to the compiler/regression suite and gate release-mode codegen.
  3. 1529 (PULSE): stored-fn env thunks emit `ptrtoint ptr
     @__unsafe_block_N`; clang 14 / Apple arm64 reject it. Emit a typed
     `i8*` operand and gate the stored-fn probe on the arm64 floor.
  4. Release-compiler parity: build `target/release/xiom.exe` and run the
     P0 probes; add a CI job (the v0.64.9 miss).
  5. m1511: transitive dependency `port.args.json` closure + two-level
     installed fontraster test (publish held).
  6. C07: v0.64.9 capture-probe memory regression (1.43-1.50 GB vs
     v0.64.4); fix or formally scope with owner approval.
- **P2 MCU first wave is not started:** target triples
  thumbv6m/thumbv7em/thumbv8m.main + riscv32imc/imac, `--freestanding`
  no_std profile, startup/linker contract, Cortex-M + riscv32 hello under
  QEMU, QEMU CI. See `docs/checklists/mcu-targets.md`.
- **Selfhost:** RD11 T3 green (19/19, 14/64, 83/83) at `6522688c9`;
  `diff_ast` 80/83. The trailing-match re-wrap reaches 83/83 but needs a
  value-returning `NkExprMatch` codegen path; do not enable the parser
  half alone. m1136 mirror + P6 shapes remain next-cycle.
- **Relay:** new compiler items `REL-20261022-1529-pulse` and
  `REL-20261022-1827-packages` (plus 1645/1615). stdlib 1640 is fixed by
  the JSON digit-loop change; bindings applied the v0.64.9 withdrawal and
  restored the v0.64.8 production slot; benchmark/orbitdb/engine/packages
  remain v0.65.0 acceptance inputs.
- **Host:** the engine incident deleted `C:\Users\lefte\.cargo`. Cargo
  works from
  `C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin`;
  prepend it to PATH. Restore the `.cargo\bin` shims for tests/CI.

*** SESSION UPDATE 2026-10-22T18:35Z (compiler lane; v0.64.9 WITHDRAWN; v0.65.0 production plan) -- READ FIRST ***
- **v0.64.9 is WITHDRAWN FROM ADOPTION.** The GitHub release is marked
  pre-release; production remains **v0.64.8**. Tag/assets stay for audit
  only. Do not pin or deploy v0.64.9. The stdlib tag/registry remain
  published but are not the production pin.
- **Why:** nested enum-pattern payloads (`Some(Number(n))`,
  `Ok(Number(n))`, `Err(E.Bad(msg))`) are flattened and bind 0; the
  m372/1451 and true-1550 regression was bisected to `d04cd3265`
  (explicit type-arg substitution; reverting restores m372 but breaks
  m1520); m1511 dependency port args are one level; C07 is still red.
  The pre-cut battery missed these because it ran the debug compiler and
  had no nested-pattern or release-compiler lock.
- **New target: v0.65.0, production, no rush.** Gate:
  `docs/RELEASE_GATE_v0.65.0.md`; execution checklist:
  `docs/checklists/v0.65.0.md`. Order: P0 correctness (nested patterns,
  safe substitution, release-compiler parity, m1511 transitive, C07),
  P1 production hardening, P2 MCU first wave (Cortex-M + riscv32, no_std,
  startup/linker contract, QEMU; `docs/checklists/mcu-targets.md`), P3
  release. No candidate/tag until the gate is green and owner GO.
- **Version:** workspace and selfhost now 0.65.0. `STDLIB_VERSION`
  remains `stdlib-v0.64.9` for development; v0.65.0 will pin
  `stdlib-v0.65.0` after the stdlib cut (which pins v0.64.8 per the
  checklist).
- **Host:** the engine incident deleted `C:\Users\lefte\.cargo`. Cargo
  works at
  `C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin`;
  restore the `.cargo\bin` shims so tests/CI do not need a lane-local
  PATH.
- **Selfhost:** RD11 T3 green (19/19, 14/64, 83/83); m1136 mirror and P6
  shapes are part of the v0.65.0 hardening, not the release blocker.
- **Lane holds:** xvector (compiler + stdlib), orbitdb (compiler pin),
  engine (C07), packages (m1511/2045), benchmark (compiler). Re-test on
  the v0.65.0 release archive before any row is closed.

*** SESSION UPDATE 2026-10-22T14:10Z (compiler lane; v0.64.9 RELEASED) -- READ FIRST ***
- **v0.64.9 IS LIVE.** Compiler tag `v0.64.9` (annotated
  `ffde5aa0f88591b40db949239340cf66a36c3e41`) on `2bcf58db5`; release run
  **39791522014 SUCCESS**; **11 assets** live; SHA256SUMS
  `51c45a61dbfebb930acc99ad8cc383234482fc0ba52961628b8f7ffdf6a9ebac`.
  Packaging dry-run 39785770942 SUCCESS (all platforms + VSIX).
- **stdlib v0.64.9:** tag `stdlib-v0.64.9` at `aa8b656`; release run
  **39785430606 SUCCESS** (2 assets); production registry publish run
  **39785430612 SUCCESS**. `xiom-std 0.64.9` is stable on
  registry.xiom-lang.org (sha256
  `6f0d3b1516c7c94fcc6fd7fc1fc34fa66025c9543456a4c2effa16142da73020`,
  compiler v0.64.8, published 2026-10-22T14:01:23Z). The automatic
  STDLIB_VERSION pin PR failed only because `2bcf58db5` already pinned
  `stdlib-v0.64.9` ("nothing to commit").
- **Contents:** m1157 BOM catalog; 2045 part 2 Str payloads; XVC-C-22 Vec
  inference; m1731 borrowed enum; m1703 stored fn values; m1326 enum
  equality; m1520 adapter substitution; 1550 EnumerateIter tuple fn
  (stdlib `aa8b656`); m365 shared dependency keys; 1511 deps port args;
  1531 dead-builder lock; m375 opt-in `XIOM_FAST_EXIT`.
- **Battery on the release commit:** feature 549/549, integration 130/130,
  stdlib 40/40, codegen lib 25/25, xiom lib 77/77, checker 214/214, parser
  109/109, plus the m1550 e2e lock. Release notes JSON verified
  (6 highlights, 0 breaking).
- **No re-cut of v0.64.8.** Post-release verification rows are in
  `release-checks/compiler-compiler-v0.64.9.md`: xvector (XVC-C-22, 1531,
  1243), orbitdb (m375 1M A/B, A6, m1157), engine (m365 re-measure, 2 GB
  cap), packages (fontraster installed build, unhold), PULSE (restore
  StreamHandler; 1703 pre-verified). Selfhost remains RD11 T3 green
  (19/19, 14/64, 83/83); m1136 mirror + P6 shapes are next-cycle work.
- The release's staging-canary dispatch (39791446653) is waiting for
  environment approval and is optional; production is already published.

*** SESSION UPDATE 2026-10-22T12:52Z (compiler lane; v0.64.9 wave-1 fixes + full battery green) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; **v0.64.9 is NOT cut**
  (no tag, no publish, no candidate JSON). Main is committed locally and
  pushed with this handoff; relay tip `fb17cdd` plus later coordination
  items.
- NEW FIXES on top of the 2026-10-21T19:05Z block:
  * m1520 / REL-20261021-1520-stdlib: `_max_via[U]` inside
    `MapIter[T,U].max` kept the literal `U`, so `%struct.Tuple__T__U` was
    compared with i64. Explicit `GenericCall` args, `type_arg` idents, and
    the index arm now substitute `current_type_map`; receiver/declared-
    return inference was added before the argument fallback (`d04cd3265`).
    `MapIter.max` + `ZipIter.find` print `max=6`, `found=2,6`; lock
    `e2e_m1520_adapter_collision`.
  * 1511 / REL-20261021-1511-packages: `package.xi` uses `deps:` but
    `extract_deps_legacy` only matched `dependencies:`, so
    `manifest.dependencies` stayed empty and dependency `port.args.json`
    never applied (installed fontraster/HarfBuzz link failure). The parser
    now accepts `deps:`/`dependencies:`, brace-matches multi-line maps, and
    splits entries quote-aware (`e45059b76`; graph 43/43). Packages are
    asked to drop the tree-relative harfbuzz workaround, re-test an
    installed fontraster build, and unhold publish.
  * m365 / REL-20261017-1355-engine: dependency C objects now use a
    consumer-neutral canonical profile (host ISA, m339 SFB disable,
    no-asm/CRT defines, PIC for shared consumers, pinned `-O2`; consumer
    `-g/-O*/LTO/sanitizer/stack-protector` excluded) (`70b380c4e`).
    Pending the engine's 117-target assimp re-measure.
  * m375 / REL-20261021-1122-orbitdb-2: measured 20k = 37 s work / <0.1 s
    exit, 200k = 527 s work / 0.04 s exit (the probe's `ms=` is SECONDS --
    `io.time_now` is C `time(0)`); the 880 s is 1M-specific. Added opt-in
    `XIOM_FAST_EXIT=1` at compile time (renames user main; wrapper flushes,
    preserves the guard-fault exit-70 path, then ExitProcess/_exit)
    (`b51ce6ec2`). Cache sizing guidance relayed; 1M A/B pending.
  * 1531 / REL-20261020-1531-xvector-2: could not reproduce the 16 GB Job
    Object build hang on main or v0.64.8 (the full 178 KB conformance file
    builds under the cap). Hardened `rtcache` to break a lock held by a
    dead pid immediately (`1fb407377`); xvector asked for the exact command
    if it persists.
  * 1243 / REL-20261021-1243-xvector: the standalone JSON numeric path
    (object, bare number, helper-shaped service extraction) passes on
    v0.64.8 and main; the shard-service context is still needed. Not the
    C-18 family in this shape.
  * 1550 / REL-20261020-1550-stdlib: compiler side ready
    (`d04cd3265`: full fn field markers, fn-field Option payload resolver,
    computed-receiver/call-site generic substitution). A 6-line
    `EnumerateIter[Int].map` specialization is relayed to stdlib
    (`REL-20261022-1152-compiler`); with it the exact probe prints
    `10,12,14`, with the current stdlib it is still red. Do NOT mark 1550
    fixed until stdlib lands and the probe is re-run.
  * Selfhost: RD11 rebase onto `d50d1436e`, T3 green (phase-5 19/19,
    phase-6 14/64, T1 83/83) at `6522688c9`. A `diff_ast` experiment
    (trailing value-shaped match -> ExprMatch) reaches 83/83 AST parity but
    regresses `examples/phase1_enum` T3 (selfhost `cgb_match` has no
    value-result path); reverted and documented at `3e9f26484`. The m1136
    selfhost mirror and the P6 shapes (Option/.len, Vec.pop, &Vec+f64,
    index arithmetic) remain.
- BATTERY (final main): feature_regression 549/549, integration 130/130,
  stdlib_tests 40/40, codegen lib 25/25, xiom lib 77/77, xiom-graph 43/43,
  checker 214/214, parser 109/109.
- v0.64.9 READINESS: the compiler tree is green. Remaining blockers are
  coordination, not codegen: stdlib's 1550 specialization, the engine's
  m365 117-target re-measure, orbitdb's m375 1M A/B, the packages
  fontraster installed re-test (1511), and the xvector 1243/1531 re-tests.
  No candidate cut/tag/publish without those replies and owner GO.

*** SESSION UPDATE 2026-10-21T19:05Z (compiler lane; 6 relay fixes + selfhost RD11 T3 GREEN) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive (tag 4d040604b, 11 assets,
  registry xiom-std 0.64.8 green); no re-cut/candidate without owner GO.
  Main has local commits on top of origin/main (not pushed yet). Relay tip
  `73210ca`.
- FIXED THIS SESSION (incoming relays first):
  * m1157 / REL-20261021-1157-orbitdb -- UTF-8 BOM hid catalog module
    headers; `read_module_header` now strips leading BOMs (`18415ede2`;
    unit `bom_prefixed_module_header_is_indexed` + e2e m1157).
  * 2045 part 2 / REL-20261019-2045-packages -- boxed Option/Result Str
    payload binding/print (`9d31321df`; pop/get/remove alias+scalar
    payload tracking, field-2 Result fallback, `local_xiom_types` on the
    boxed arm; e2e m2045).
  * XVC-C-22 / REL-20261021-1103-xvector -- Vec element inference prefers
    the callee's DECLARED return over the first argument (`d38cff2a8`).
    `pq_train`'s `part` was registered as Vector (40-byte stride), which
    corrupted the codebook and the guard heap -> second-call AV/ud2.
    Probe 20/20 stable; centroid checksum 66254.9604248703 identical for
    first/second x5; v0.64.8 varied/crashed. Runtime + IR locks.
  * m1731 / REL-20261021-1731-stdlib -- `match &enum` loads the tag instead
    of comparing the alloca pointer (`ace095744`); borrowed Str payloads
    bind (e2e m1731; Number/Bool controls).
  * m1703 / REL-20261021-1703-pulse -- stored fn values keep signature and
    aggregate return; `Vec[Handler]` aliases expand and bare fn refs get
    the `__fnwrap` env thunk (`c9b1f1009`); StreamHandler shape returns
    (4,7) direct/module/Vec; PULSE probe rc 0 (e2e m1703).
  * m1326 / REL-20261020-1326-stdlib -- struct field equality on an enum
    without a derived `.eq` compares the discriminant (`d50d1436e`);
    log.json+serialize compiles/runs (e2e m1326).
- SELFHOST: `selfhost-phase-4-codegen` rebased onto `d50d1436e` (53 commits,
  no conflicts; `selfhost/` byte-identical to `589fc67f7`). T3 tier 3:
  phase-5 scalar 19/19, phase-6 corpus 14/64, T1 green 83/83. RD11
  `6522688c9`. The separate `diff_ast` check is 80/83 (Rust `Tail` vs
  selfhost `Stmt` on phase1_enum/phase1_hardening/benchmark_selfhost),
  unchanged from the old tip and not part of the T3 meter.
- RELAYS: 1157, 2045, 1103, 1731, 1703, 1326 marked fixed with evidence.
  1550-stdlib acked/diagnosed: generic-STRUCT field erasure (`MapIter` is
  emitted as `{fn()->Option, fn(i64)->i64}`); needs a generic-struct pass or
  a concrete `EnumerateIter.map` specialization, not a one-line ABI fix.
- BATTERY: feature 549/549, integration 130/130, stdlib 40/40, codegen lib
  23/23, checker 214/214, parser 109/109. New locks: m1157, m2045,
  XVC-C-22 (runtime+IR), m1731, m1703, m1326.
- NEXT: 1) 1550 generic-struct erasure; 2) mirror m1136 on the selfhost
  block-arm match payload path; 3) the 3 `diff_ast` Tail/Stmt parser drifts;
  4) 1531-xvector-2 Job Object build hang; 5) m375 drop/exit reclaim;
  6) m365 caller-side dependency-vs-consumer flag split. No candidate cut
  without owner GO.

*** SESSION UPDATE 2026-10-21T11:46Z (compiler lane; CONTEXT HANDOFF -- incoming relays triaged, m1136 fixed, selfhost T3 83/83) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive (tag 4d040604b, 11 assets,
  registry xiom-std 0.64.8 green); no re-cut/candidate without owner GO.
  Main code tip `478e06f8b` (m1136) + this handoff docs commit; origin synced.
  Relays `f22a7f6`.
- FIXED THIS SESSION: m1136 block-arm Vec match binding (`478e06f8b`; lock
  `e2e_m1136_vec_match_binding`, via_let=3); 1122-orbitdb global Vec.pop
  shrink (already fixed on main by m2057 `029595080`; v0.64.8 predates; exact
  repro 2/1/0, relay fe2ab3d); m0717 arm64 field-0 GEP (`aecad8c6c`; lock
  `e2e_m0717_module_vec_ginit_typed_ir`); 2045 part 1 (`05e3ab3bc`;
  Vec.pop/remove Option[T]); m1626 single match seed (`c2dc54a24`), m365
  per-key cache lock (`306ace857`), m2343 (`2f887bcf1`), m1626 (`626dfbc65`),
  m2100-W1 (`dea6ceee8`), m2057 (`029595080`), m376/m377 (`f915fb759`).
  Full battery green: feature 549/549, integration 130/130, stdlib 40/40,
  codegen lib 23/23, checker 213/213, parser 109/109; all recent e2e locks.
- SELFHOST RD10: T3 was GREEN 83/83 (phase-5 19/19, phase-6 14/64) on branch
  tip `589fc67f7`, which is a LOCAL rebase onto `c2dc54a24` and NOT
  force-pushed. IMPORTANT: that T3 predates m1136 (`478e06f8b`), which changes
  block-arm match IR; rebase selfhost onto current main and re-run T3 before
  continuing P6 (a selfhost mirror may be needed).
- TOP OPEN:
  1) 2045 Str payload: `Some(s)` / `Err(msg)` from a boxed Option/Result
     still prints pointer bits (no Str XIOM type record on that match
     binding). m1136 fixed the aggregate Vec block-arm result only. Next:
     record/unbox Str on the match binding (mirror `bind_is_payload_xiom`),
     add a repo fixture; probes were in `%TEMP%\kilo\e2045`.
  2) XVC-C-22 (REL-20261021-1103-xvector): ACKED, reproduced. Exact
     `E:\xiom-projects\xiom-xvector\docs\repro\xv-pq-second-call\probe.xi`
     prints `built N`, `first ok`, then the second pq_train AVs. Trigger is
     TRAINING WORK per call (m/ksub/iters), not corpus size: 100 vec +
     (m8,k2,i1) or (m1,k16,i8) crash; (m1,k2,i1) passes at 10/100/2000.
     Push IR is correct (40-byte Vector memcpy); no frees in pq_train; guard
     heap only in stdlib unsafe blocks; instrumented/ASAN builds PASS
     (layout/heap-sensitive). Need a smaller deterministic repro or crash
     dump (PC/stack). A slot-growth experiment was REVERTED -- do not land
     speculative slot changes; P4 stays blocked.
  3) 1326-stdlib: v0.64.8 log+serialize IR `icmp on %struct.LogLevel`
     (Map.insert_LogEntry_JsonValue).
  4) 1550-stdlib: tuple-returning fn-value ABI (`EnumerateIter.map` garbage).
  5) 1531-xvector-2: build hangs under a Job Object with 16 GB limits.
  6) m375: drop elaboration/exit reclaim. New orbitdb data: 1M PASSES with a
     65,536-frame cache (2,494.7MB, exact); remaining is ~880 s exit teardown
     and cache sizing. `read_at_into` already fixed the read path engine-side.
  7) m365: per-key lock landed; only the caller-side dependency-vs-consumer
     flag split remains, blocked on the engine's 117-target key-equality
     re-measure and the two key inputs/args.
- OPERATIONAL: E: can stall in `Wait PageIn` under other lanes' compiles. Use
  a C:-isolated copy of `xiom.exe` + `lib/` with private HOME, XIOM_STDLIB,
  `--no-cache` (this session's reliable setup). Do NOT kill all `xiom`
  processes; other lanes run compilers.
- NEXT: 2045 Str binding/print; XVC-C-22 repro/dump; 1326, 1550, 1531;
  selfhost rebase + T3 + P6 shapes (Option/.len, Vec.pop, &Vec+f64, index
  arithmetic); m375 design; m373/m374; MCU 1220; m365 flag split. No
  candidate cut without owner GO.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-21T11:46Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-21T11:46Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays. Pull xiom-relays first and process new items
addressed to compiler before starting.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main is origin-synced; code tip
`478e06f8b` (m1136 block-arm Vec match binding). Other recent fixes: 2045
part 1 `05e3ab3bc` (Vec.pop/remove Option[T]); m1626 single match seed
`c2dc54a24`; m0717 arm64 field-0 GEP `aecad8c6c`; m365 per-key cache lock
`306ace857`; m2343 `2f887bcf1`; m1626 `626dfbc65`; m2100-W1 `dea6ceee8`;
m2057 `029595080`; m376/m377 `f915fb759`. Relays `f22a7f6`. Orbitdb global
Vec.pop shrink is fixed on main (m2057; v0.64.8 predates it). Selfhost branch
`589fc67f7` is a LOCAL rebase (not force-pushed) onto `c2dc54a24`; its T3 was
83/83 (phase-5 19/19, phase-6 14/64) BEFORE m1136. Rebase onto current main
and re-run T3 first; m1136 changes block-arm match IR and may need a selfhost
mirror.

Top open:
1) 2045 Str payload: `Some(s)`/`Err(msg)` from a boxed Option/Result still
   prints pointer bits. m1136 fixed aggregate block-arm Vec results only.
   Record/unbox the Str XIOM type on that match binding (mirror
   bind_is_payload_xiom) and add a repo fixture.
2) XVC-C-22 (REL-20261021-1103-xvector): acked/reproduced; exact pq probe AVs
   on the second pq_train. Trigger is training work per call, not corpus size;
   instrumented/ASAN builds pass. Need a smaller deterministic repro or crash
   dump. Do NOT land speculative Vec slot/growth changes (an experiment was
   reverted).
3) 1326-stdlib: log+serialize `icmp on %struct.LogLevel`.
4) 1550-stdlib: tuple-returning fn-value ABI (`EnumerateIter.map` garbage).
5) 1531-xvector-2: build hang under a 16 GB Job Object.
6) m375: drop elaboration/exit reclaim. 1M passes with a 65,536-frame cache
   (2,494.7MB); remaining ~880 s exit teardown + cache sizing; read path
   fixed engine-side.
7) m365: only the caller-side dependency-vs-consumer flag split remains;
   blocked on the engine's 117-target re-measure + two key inputs/args.

Priority: 1) 2045 Str binding/print; 2) XVC-C-22 repro/dump; 3) 1326, 1550,
1531; 4) selfhost rebase + T3 + P6 shapes (Option/.len, Vec.pop, &Vec+f64,
index arithmetic); 5) m375 design, m373/m374, MCU 1220. No candidate cut
without owner GO.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere. If E: stalls in PageIn, run a C:-isolated xiom.exe+lib copy with a
private HOME; do not kill other lanes' xiom processes.

*** SESSION UPDATE 2026-10-21T14:00Z (compiler lane; m0717 fixed; SELFHOST T3 83/83 GREEN) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut/candidate without
  owner GO. Main `478e06f8b` (m1136) on `05e3ab3bc` (2045 part 1),
  `c2dc54a24` (m1626 single match seed), `aecad8c6c` (m0717 field-0 GEP),
  `98ce2bbd0`, `8bfd4235d`, `306ace857`, `2f887bcf1`, `626dfbc65`,
  `dea6ceee8`, `029595080`, `f915fb759`; relays `f22a7f6`.
- ORBITDB GLOBAL POP FIXED on main (029595080, REL-20261021-1122-orbitdb):
  exact repro now prints `before=2`, `after_pop=1 has=true`,
  `after_loop=0 loops=1` (v0.64.8 predates m2057 and printed 2/2/2). Relay
  fe2ab3d; per-struct-field/reset workaround no longer needed.
- m1136 FIXED (`478e06f8b`, REL-20261021-1136-orbitdb): `infer_match_llvm_type`
  only applied the scrutinee-payload inference to DIRECT arms, so a
  BLOCK-tailed `Ok(x) => { x }` typed the result slot i64 and stored the Vec's
  data pointer; `let v = match ...; v.len()` returned 1 (Str.len on the data
  pointer) instead of 3. Block arms now use the same payload fallback; lock
  `e2e_m1136_vec_match_binding` (`via_let=3 via_var=3 direct=3`).
- 2045 PARTIAL (`05e3ab3bc`, REL-20261019-2045-packages): the builtin
  `Vec.pop` (functions + methods tables) and `Vec.remove` returned a BARE
  `Option`; both now carry `T` and return `Option[T]`, so unannotated
  `v.pop()` types from `Vec[E]` (traced: `Vec[S] -> Option[S]`,
  `Vec[Result[Str, Str]] -> Option[Result[Str, Str]]`). REMAINING: the push
  is correct (IR memcpy's the full 24-byte Result); `Vec.pop` BOXES
  non-scalar payloads into generic `%struct.Option { i64 tag, i64 boxed_ptr }`
  and the match-arm rebind treats the boxed pointer as the value. Explicit
  annotations do not help. m1136 fixed the AGGREGATE (Vec) block-arm result
  slot; the remaining 2045 path is the Str payload binding/print
  (`Some(s)`/`Err(msg)` still lacks the Str XIOM type record).
- XVC-C-22 ACKED, NOT FIXED (relay ec7d832, REL-20261021-1103-xvector): exact
  pq probe reproduces on main (`first ok`, second pq_train AV). Reduction:
  10/100/2000 vec with (m1,k2,i1) pass; 100 vec with (m8,k2,i1) or
  (m1,k16,i8) crash on the SECOND call. Trigger is training work per call,
  not corpus size. Vec[Vector] push is correct (40-byte memcpy), no frees in
  pq_train, guard heap only in stdlib unsafe blocks. Instrumented/ASAN builds
  PASS -> layout/heap-sensitive. A slot-growth experiment improved the
  10-vec case but not the full one and would keep 8-byte slots permanently;
  REVERTED, no XVC-C-22 compiler change landed. Need a smaller deterministic
  repro or a crash dump (PC/stack). P4 stays blocked.
- m0717 FIXED (`aecad8c6c`, REL-20261021-0717-pulse): `emit_vec_load_fields`
  now loads field 0 through its field GEP, not the struct pointer. Typed-
  pointer LLVM (clang 14 arm64 / Apple clang) accepts the module-scope Vec
  init IR; opaque-pointer builds are unchanged at runtime. Lock
  `e2e_m0717_module_vec_ginit_typed_ir` emits arm IR and asserts every ginit
  data load reads a `%struct.Vec` GEP. Selfhost mirror: `930adab1f`.
- m1626 PARITY (`c2dc54a24`): the `Expr::Match` wrapper seeded the result slot
  and the inner `Stmt::Match` seeded it again; the wrapper seed is removed
  (the statement match owns it, before the scrutinee). `phase1_enum` Rust IR
  is 194 lines, matching selfhost.
- SELFHOST RD10: **T3 GREEN 83/83** (tier 3, 83 files, 225 s; phase-5 scalar
  19/19, phase-6 corpus **14/64** byte-exact). Branch tip `589fc67f7` is
  rebased onto `c2dc54a24` LOCALLY; NOT force-pushed. Next P6 shapes:
  Option/`.len`, `Vec.pop`, `&Vec`+f64, index arithmetic.
- NEW OPEN ITEMS (overnight): REL-20261020-1326-stdlib (log+serialize
  `icmp on %struct.LogLevel`); REL-20261020-1550-stdlib (tuple-returning
  fn-value ABI: `EnumerateIter.map` garbage); REL-20261020-1531-xvector-2
  (build hangs under a Job Object with 16 GB limits). m375 has new
  high-severity orbitdb evidence: read path fixed by `read_at_into`
  engine-side, residual page encode/write + Vec-drop; 1M is cap-killed at
  4.17 GB/~435k keys (9.6 KB/key, superlinear) -- drop elaboration remains
  the proper fix.
- m365: per-key lock landed; only the caller-side dependency-vs-consumer
  flag split remains, blocked on the engine's 117-target re-measure and the
  two key inputs/args.
- VERIFICATION: feature_regression 549/549, integration 130/130,
  stdlib_tests 40/40, codegen lib 23/23, parser 109/109, checker 213/213;
  m0717/m1626/m2057/m2100/m2343/m337/m371/m372/m376/m377 locks green;
  float/other selfhost suites unchanged.
- OPERATIONAL: E: can stall in `Wait PageIn`; use a C:-isolated toolchain
  copy with private HOME/XIOM_STDLIB/--no-cache. Do NOT kill all xiom
  processes (other lanes compile).
- QUEUE: 2045 Str payload binding/print first, then XVC-C-22 (needs a
  smaller repro/dump), 1326-stdlib, 1550-stdlib, 1531-xvector; m375
  drop-elaboration design + 1M/200k probes; selfhost P6 shapes; m373/m374;
  MCU 1220; m365 flag split on engine data. No candidate cut without owner
  GO.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-21T14:00Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-21T14:00Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main `478e06f8b` (m1136 block-arm
Vec match binding) on `05e3ab3bc` (2045 part 1: Vec.pop/remove Option[T]),
`c2dc54a24` (m1626 single match seed), `aecad8c6c` (m0717 field-0 GEP) and the
earlier fixes. Relays `f22a7f6`. Also fixed on main: orbitdb global Vec.pop
shrink (029595080; exact repro 2/1/0, v0.64.8 predates) and m1136. SELFHOST
RD10: T3 is GREEN 83/83 (phase-5 19/19, phase-6 14/64); branch `589fc67f7` is
a local rebase, not force-pushed.

2045 is PARTIAL: pop typing is fixed, and m1136 fixed the AGGREGATE (Vec)
block-arm match result slot. The remaining 2045 path is the Str payload
binding/print: `Some(s)` / `Err(msg)` from a boxed Option/Result still lacks
the Str XIOM type record, so it prints pointer bits. Push IR is correct
(24-byte Result memcpy). XVC-C-22 is ACKED, not fixed: the exact pq probe
still AVs on the second call; reduced cases show the trigger is training work
per call; instrumented/ASAN builds pass (layout-sensitive); need a smaller
repro or crash dump before touching code.
Other open: 1326-stdlib (log+serialize `icmp on %struct.LogLevel`), 1550-stdlib
(tuple fn-value ABI `EnumerateIter.map`), 1531-xvector-2 (Job Object 16 GB
hang). m375: read path fixed engine-side; residual page encode/write +
Vec-drop; 1M cap-killed at 4.17 GB/~435k keys -- drop elaboration design
needed.

Priority: 1) 2045 Str payload binding/print, then XVC-C-22 (needs repro/dump),
1326-stdlib, 1550-stdlib, 1531-xvector-2. 2) m375 drop-elaboration design +
200k/1M probes. 3) Selfhost P6 shapes: Option/.len, Vec.pop, &Vec+f64, index
arithmetic. 4) m373/m374, MCU 1220; m365 flag split on engine data. 5) Wait
for lane re-verify; no candidate cut without owner GO.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere. If E: stalls in PageIn, use a C:-isolated toolchain copy with a
private HOME; do not kill other lanes' xiom processes.

*** SESSION UPDATE 2026-10-20T20:34Z (compiler lane; m365 per-key lock LANDED; gate-9 reds cleared) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut/candidate without
  owner GO. Main `8bfd4235d` (m1626 follow-up) on top of `306ace857` (m365
  lock), `2f887bcf1` (m2343), `626dfbc65` (m1626), `dea6ceee8` (m2100-W1),
  `029595080` (m2057), `f915fb759` (m376/m377); relays `fc2b563`.
- m365 PARTIAL+ (`306ace857`, REL-20261017-1355-engine): `rtcache::
  ensure_objects` now takes an atomic per-key `.lock` around the MISS build,
  re-checks marker+objects under the lock (waiters HIT when the peer
  finishes), removes `.ready` before rebuilding and publishes it only after
  every object is renamed; stale locks break after 30 min. HIT still requires
  EVERY object. Tests `key_lock_is_exclusive` (8 threads, max concurrency 1)
  and `concurrent_dependency_objects_share_one_key` (4 callers, same complete
  list); `cargo test -p xiom --lib` 75/75. STILL OPEN: caller-side
  dependency-vs-consumer flag split -- blocked on the engine's 117-target
  key-equality re-measure and the two key inputs/args.
- m1626 FOLLOW-UP (`8bfd4235d`, relay fc2b563): the selfhost rebase exposed
  `alloca void` for a Unit-valued match EXPRESSION (m1626 made a
  semicolon-less value-shaped match an expression). `infer_match_llvm_type`
  now ignores `void`/`()` arm types (all-Unit falls back to i64; mixed picks
  the real type). Minimal repro + `unit_match_expr` added to the m1626 lock;
  this unblocked the selfhost skeleton compile.
- m2100-W1/m1626/m2343/m2057 are fixed (see the 19:11Z block below for the
  full evidence and locks); stdlib gate-9 reds are cleared pending reporter
  re-verify.
- m375 remains MEASURED/acked (Vec drop elaboration; no fix): direct-reader
  workaround, stdlib `read_at_into` wishlist `REL-20261020-1302-compiler`;
  compiler proper fix needs owned-payload/scope-exit drop design.
- SELFHOST RD10: rebase onto `8bfd4235d` is CLEAN (50/50) and LOCAL (not
  force-pushed). Skeleton compiles after the m1626 follow-up. T3 (tier 3,
  83 files, 171 s): **82/83 green**; phase-5 19/19, phase-6 13/64 (14 ported).
  Sole failure `examples/phase1_enum.xi` T2 line 134: rust `store i64 0`
  (Expr::Match result-slot seed) vs selfhost `load`. Selfhost must port that
  seed for the trailing-match-as-expression shape. RD10 docs committed
  `d20d14156` on the branch.
- OPERATIONAL: E: can stall in `Wait PageIn` under other lanes' compiles. Use
  a C:-isolated copy of `xiom.exe` + `lib/`, private `HOME`, `XIOM_STDLIB`,
  `--no-cache`. Do NOT kill all `xiom` processes; other lanes run compilers.
- QUEUE: selfhost -- port the match-slot seed, re-run T3 (expect 83/83), then
  Option/.len, Vec.pop, &Vec+f64, index arithmetic; m373 profiling/m374; MCU
  1220; m365 flag split when the engine reports; m375 design only when the
  owner prioritizes it.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T20:34Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T20:34Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main `8bfd4235d` (m1626 Unit
match-slot follow-up) on top of `306ace857` (m365 per-key cache lock),
`2f887bcf1` m2343, `626dfbc65` m1626, `dea6ceee8` m2100-W1, `029595080` m2057,
`f915fb759` m376/m377. Relays `fc2b563`. stdlib gate-9 reds are fixed pending
reporter re-verify; m365 still needs the engine's 117-target key-equality
re-measure before the flag split; m375 is measured/no-fix.

SELFHOST RD10: branch `selfhost-phase-4-codegen` rebased cleanly onto
`8bfd4235d` (local, not force-pushed); skeleton compiles. T3 is 82/83:
phase-5 19/19, phase-6 13/64. Sole failure `phase1_enum.xi` T2 line 134 --
the selfhost match path must emit the Rust `store i64 0` result-slot seed for
the trailing-match-as-expression shape. RD10 docs are `d20d14156`.

Priority: 1) Selfhost: port the match-slot seed, re-run T3 (expect 83/83),
then continue Option/.len, Vec.pop, &Vec+f64, index arithmetic. 2) m373
profiling/XIOM_TIMINGS, m374 design, MCU 1220. 3) m365 flag split when the
engine reports the two key inputs/args. 4) Wait for lane verification of the
v0.64.8 check rows and the re-verify rows; no candidate cut without owner GO.
m375 design only when the owner prioritizes it.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere. If E: stalls in PageIn, run a C:-isolated toolchain copy with a
private HOME; do not kill other lanes' xiom processes.

*** SESSION UPDATE 2026-10-20T19:11Z (compiler lane; stdlib gate-9 reds CLEARED: m2100-W1 + m1626 + m2343) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut/candidate without
  owner GO. Main `2f887bcf1` (m2343) on top of `626dfbc65` (m1626),
  `dea6ceee8` (m2100-W1), `029595080` (m2057), `f915fb759` (m376/m377);
  relays `7955855`.
- m2100-W1 FIXED (`dea6ceee8`, REL-20261016-2100-stdlib): instance-field
  receiver inference normalized the declared field type exact ->
  `xiom.<qualified>` -> deterministic leaf key before method dispatch.
  Stdlib witnesses `w1546_field.xi` (alias), `w1546_field_full.xi` (full
  path), `w1546_field_bare.xi` (control) all rc 0; lock
  `e2e_m2100_alias_field_type` (`field_type_ok`). The bare-name workaround in
  `xiom/net/sse.xi` can be reverted.
- m1626 FIXED (`626dfbc65`, REL-20261011-1626-stdlib): a semicolon-less
  trailing `match` whose arms are VALUE-shaped is now `Expr::Match` (the block
  value); an explicit `match ...;` keeps discard semantics, and matches with
  return/loop arms stay statements. The checker also clears `last_expr_ty` at
  every statement (the preceding `vec.push(x)` was parsed as a tail
  expression and its Unit type leaked). Probe rc 0; lock
  `e2e_m1626_trailing_match_after_push` (includes the `match ...;` discard
  control). The `return match` workaround in `xiom/serialize/json.xi` can be
  reverted.
- m2343 FIXED (`2f887bcf1`, REL-20261015-2343-stdlib):
  `infer_call_return_xiom` now unwraps `unsafe { ... }` / `{ ... }` to the
  block tail, so `let a = unsafe { ext_i32() }` records Int32 + signedness and
  the comparison sign-extends (pre-fix `zext`: -1 -> 4294967295). Socket
  witness rc 0; lock `e2e_m2343_inline_i32_return` (deterministic strcmp
  negative/positive/declared forms -> `i32_neg_ok`). The 27 stdlib
  annotations remain harmless.
- m2057 FIXED (`029595080`, REL-20261019-2057-engine): fn-type alias
  transparency (checker + codegen, including struct fields) + module-global
  `Vec.pop`; locks `e2e_m2057_alias_params` / `e2e_m2057_global_pop`.
- VERIFICATION: parser 109/109, checker 213/213, feature_regression 549/549,
  integration 130/130, stdlib_tests 40/40, codegen lib 23/23; locks
  m1626/m2057/m2100/m2343/m337/m371/m372/m376/m377 green;
  `p_wave196_small_gaps.xi` rc 0. stdlib gate-9 reds 1626/2343/2100-W1 are
  now all fixed; reporter re-verify is the next step.
- m375 remains MEASURED/acked (Vec drop elaboration; no fix): direct-reader
  workaround, stdlib `read_at_into` wishlist `REL-20261020-1302-compiler`;
  compiler proper fix needs owned-payload/scope-exit drop design.
- OPERATIONAL: E: can stall in `Wait PageIn` under other lanes' compiles. Use
  a C:-isolated copy of `xiom.exe` + `lib/`, private `HOME`, `XIOM_STDLIB`,
  `--no-cache`. Do NOT kill all `xiom` processes; other lanes run compilers.
- QUEUE: m365 caller flag-split/per-key lock (engine `REL-20261017-1355`;
  PARTIAL at 51cf364c8 -- ready-marker + canonical paths landed; BLOCKED on
  the engine's 117-target key-equality re-measure before the flag split);
  selfhost rebase + Option/.len, Vec.pop, &Vec+f64, index arithmetic; m373
  profiling/m374; MCU 1220. m375 design only when the owner prioritizes it.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T19:11Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T19:11Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main `2f887bcf1`: m2343 inline
i32 extern sign-extension; `626dfbc65`: m1626 trailing match value; `dea6ceee8`:
m2100-W1 qualified field types; `029595080`: m2057 fn-type aliases + global
Vec.pop; `f915fb759`: m376/m377. Relays `7955855` mark all four fixed (plus
m376/m377). stdlib gate-9 reds 1626/2343/2100-W1 are cleared pending reporter
re-verify; Vec4f lane math is `REL-20261020-1242-compiler`; read_at_into is
`REL-20261020-1302-compiler`.

m375 is measured, NOT fixed: read_at 10k x 4 KiB = 304 MB; Vec[UInt8] churn
alone = 317 MB/10k; same churn in a helper = 4.6 MB flat; a read helper
returning only d.len() still grows 310 MB. Root cause is Vec drop elaboration
(match-bound Result Vec payloads are not drop candidates; loop locals drop only
at function return/last binding), not io.read_at or the arena. Orbitdb has the
direct-reader workaround; the compiler proper fix needs owned-payload/
scope-exit drop design first (no unsafe drop patch).

Priority: 1) m365 caller flag-split/per-key lock (engine `REL-20261017-1355`;
PARTIAL at 51cf364c8, BLOCKED on the engine's 117-target key-equality
re-measure before the flag split). 2) Selfhost: rebase 096bad953, re-run T3,
continue Option/.len, Vec.pop, &Vec+f64, index arithmetic. 3) Performance:
m373 profiling/XIOM_TIMINGS, m374 design, MCU 1220. 4) Wait for lane
verification of the v0.64.8 check rows and the four re-verify rows; no
candidate cut without owner GO. m375 design only when the owner prioritizes
it.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere. If E: stalls in PageIn, run a C:-isolated toolchain copy with a
private HOME; do not kill other lanes' xiom processes.

*** SESSION UPDATE 2026-10-20T18:44Z (compiler lane; m2057 FIXED; m375 measured) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut/candidate without
  owner GO. Main `029595080` (m2057) on top of `f915fb759` (m376 `9b6b3b59f`
  + m377 `f5525e9f6`); relays `faa3930`.
- m2057 FIXED at `029595080` (REL-20261019-2057-engine):
  (a) checker alias transparency at the local-fn, struct-fn-field and
  general callee paths, on both binary operands, and for nested-module alias
  keys; (b) codegen: alias fn params use the i64 closure-env ABI (not a raw
  `i64 (i64)*` pointer), fn-marker stores resolve the destination alias
  before wrapping, fn-field call detection resolves the field alias;
  (c) Vec.pop uses `resolve_vec_receiver_ptr`, so module-global/index/handle
  receivers mutate in place and locals keep the round trip.
  Locks: `e2e_m2057_alias_params` (`alias_ok 209`; same-module, qualified,
  struct-field literal + reassignment, qualified Int alias) and
  `e2e_m2057_global_pop` (`pop_ok`; global/local/field/&mut).
  IMPORTANT: the literal `type F` snippet is MASKED on v0.64.8 by the
  single-uppercase generic-param fallback; multi-letter, qualified, and
  struct-field aliases were the real repro. Engine workaround can be reverted.
- VERIFICATION this session: checker 213/213, feature_regression 549/549,
  integration 130/130, stdlib_tests 40/40, codegen lib 23/23,
  m249/m283/m312/m326/m337/m371/m372/m376/m377/m37_bug55 locks green,
  `p_wave196_small_gaps.xi` rc 0.
- m375 remains MEASURED/acked (Vec drop elaboration; no fix): direct-reader
  workaround, stdlib `read_at_into` wishlist `REL-20261020-1302-compiler`,
  compiler proper fix needs owned-payload/scope-exit drop design.
- OPERATIONAL: the E: toolchain can stall in `Wait PageIn` under other lanes'
  compiles. A C:-isolated copy of `xiom.exe` + `lib/`, with a private `HOME`,
  `XIOM_STDLIB`, and `--no-cache`, runs reliably. Do NOT `Get-Process xiom |
  Stop-Process` -- other lanes run compilers; track only your own PID.
- QUEUE: stdlib reds 1626/2343/2100-W1, m365; selfhost rebase + shapes; m373
  profiling/m374; MCU 1220. m375 proper-fix design only when the owner
  prioritizes it (no unsafe drop patch).

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T18:44Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T18:44Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main `029595080` fixes m2057
(fn-type alias transparency in checker + codegen, module-global Vec.pop;
locks `e2e_m2057_alias_params` / `e2e_m2057_global_pop`). Earlier fixes:
`9b6b3b59f` m376 (Map/Set helper AV), `f5525e9f6` m377 (scalar-pointee
inference + call-site pointer ABI). Relay `faa3930` marks m2057 fixed; m376
`26bfcd9` and m377 `d75f0de` fixed; Vec4f lane-math follow-up
`REL-20261020-1242-compiler`; read_at_into wishlist
`REL-20261020-1302-compiler`.

m375 is measured, NOT fixed: read_at 10k x 4 KiB = 304 MB; Vec[UInt8] churn
alone = 317 MB/10k; same churn in a helper = 4.6 MB flat; a read helper
returning only d.len() still grows 310 MB. Root cause is Vec drop elaboration
(match-bound Result Vec payloads are not drop candidates; loop locals drop only
at function return/last binding), not io.read_at or the arena. Orbitdb has the
direct-reader workaround; the compiler proper fix needs owned-payload/
scope-exit drop design first (no unsafe drop patch).

Priority: 1) stdlib reds 1626/2343/2100-W1, then m365. 2) Selfhost: rebase
096bad953, re-run T3, continue Option/.len, Vec.pop, &Vec+f64, index
arithmetic. 3) Performance: m373 profiling/XIOM_TIMINGS, m374 design, MCU
1220. 4) Wait for lane verification of the v0.64.8 check rows; no candidate
cut without owner GO. m375 design only when the owner prioritizes it.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere. If E: stalls in PageIn, run a C:-isolated toolchain copy with a
private HOME; do not kill other lanes' xiom processes.

*** SESSION UPDATE 2026-10-20T16:03Z (compiler lane; m375 MEASURED -- Vec drop elaboration, not the arena) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut/candidate without
  owner GO. Main `f915fb759` (m376 `9b6b3b59f` + m377 `f5525e9f6`); relays
  `5a4bf9f`.
- m375 MEASUREMENT (v0.64.8 Windows, 64 MiB file, 50 ms WorkingSet sampling):
  `io.read_at` 10k x 4 KiB = 304 MB (~31 KB/read); 10k x 64 B = 10.5 MB;
  2k x 64 KiB = 926 MB (~463 KB/read). Differential: Vec[UInt8]
  `with_capacity(4096)` + 4096 pushes alone (NO I/O) = 317 MB/10k (~32 KB per
  iteration); the SAME churn inside a helper called 10k times = 4.6 MB flat;
  raw malloc/free 100k x 4 KiB = 2.6 MB flat. A read helper that returns only
  `d.len()` still grows 310 MB/10k.
- ROOT CAUSE: Vec drop elaboration. A match-bound `Result[Vec[...], ...]`
  payload (the `read_at` shape) is never registered as an owned drop
  candidate; a loop-local Vec is dropped only at function return and only for
  the LAST binding. `with_capacity` also allocates `cap * 8` bytes even for
  Vec[UInt8] (elem_size 1) -- an 8x multiplier, not the root cause. The arena,
  `xiom_read_at`, and the C buffer are NOT implicated; no arena change made.
- m375 PATH: orbitdb workaround = the direct byte-level page reader (their
  fallback); stdlib wishlist `REL-20261020-1302-compiler` asks for
  `read_at_into(path, offset, &mut Vec[UInt8], len)` (caller buffer, no fresh
  Vec). Compiler proper fix = owned-payload/scope-exit drop elaboration;
  design first, do not land a use-after-free-prone patch.
- m377 FULL CHECK: with the stdlib working-tree lane fix (element-based
  `data + index` in `Vec4f.get`/`set`; uncommitted at doc time) the full
  repro prints `get1=2 get3=4`, rc 0, with no unknown-type warnings. The
  compiler lock remains green; stdlib still owns committing that change.
- RELAYS: `5a4bf9f`. m375 acked with the trace (1222) and the perf addendum
  1236 acked: R2/R3 50k rows are PRE-RECLAIM. m376 fixed (`9b6b3b59f`, commit
  `26bfcd9`); m377 fixed (`f5525e9f6`, commit `d75f0de`) + stdlib Vec4f
  lane-math follow-up `REL-20261020-1242-compiler`.
- VERIFICATION already green this session: feature_regression 549/549,
  integration 130/130, stdlib_tests 40/40, codegen lib 23/23, plus the
  m249/m283/m326/m337/m371/m372/m376/m377/m37_bug55 locks.
- QUEUE: engine REL-20261019-2057; stdlib reds 1626/2343/2100-W1; m365;
  selfhost rebase + Option/.len, Vec.pop, &Vec+f64, index arithmetic; m373
  profiling/m374; MCU 1220. v0.64.8 check rows remain reporter-side.
- CONSTRAINTS unchanged: rustc-first study; max 3-4 heavy sessions; watch
  memory; no stdlib edits except via relay; no release re-cut without owner
  GO; `--no-cache` for shared catidx/engine builds.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T16:03Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T16:03Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut/candidate without owner GO. Main `f915fb759` has:
`9b6b3b59f` m376 (clear per-mono param/ref tracking; Map/Set helper AV, lock
`e2e_m376_map_set_helpers`) and `f5525e9f6` m377 (scalar-pointee inference +
call-site pointer ABI, lock `e2e_m377_ptr_scalar_lane`; the stdlib working
tree already has the element-based `data + index` lane fix, uncommitted, and
the full Vec4f repro is green). Relay `5a4bf9f`:
m375 is ACKED with the measured trace (and the 1236 perf addendum: R2/R3 rows
are pre-reclaim); m376/m377 fixed; stdlib Vec4f lane math
`REL-20261020-1242-compiler`; stdlib `read_at_into` wishlist
`REL-20261020-1302-compiler`.

m375 is measured, NOT fixed: read_at 10k x 4 KiB = 304 MB; Vec[UInt8] churn
alone = 317 MB/10k; the same churn in a helper = 4.6 MB flat; a read helper
returning only d.len() still grows 310 MB. Root cause is Vec drop elaboration
(match-bound Result Vec payloads are not drop candidates; loop locals drop only
at function return/last binding), not io.read_at or the arena. Orbitdb has the
direct-reader workaround; compiler proper fix needs owned-payload/scope-exit
drop design first.

Priority: 1) Engine REL-20261019-2057, then stdlib reds 1626/2343/2100-W1,
m365; m375 proper-fix design when the owner prioritizes it (no unsafe drop
patch). 2) Selfhost: rebase 096bad953, re-run T3, continue Option/.len,
Vec.pop, &Vec+f64, index arithmetic. 3) Performance: m373 profiling/XIOM_TIMINGS,
m374 design, MCU 1220. 4) Wait for lane verification of the v0.64.8 check rows;
no candidate cut without owner GO.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere.

*** SESSION UPDATE 2026-10-20T15:55Z (compiler lane; m376 FIXED; m377 already fixed) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut and no candidate cut
  without owner GO.
- MAIN: `9b6b3b59f` fix(codegen) m376 -- clear
  `param_locals`/`ref_params`/`mut_ref_params` per mono body. Root cause: the
  mono path never reset them, so `map_get_or`'s `key: &K` left
  `ref_params["key"]`; the later `map_insert_if_absent`'s by-value `key`
  compiled as a reference at `m.insert(key, value)` (`inttoptr 2` + load) ->
  0xC0000005. The Set pair leaked the same way. Lock
  `e2e_m376_map_set_helpers`. Previous `f5525e9f6` is the m377 compiler fix
  (scalar-pointee inference + call-site pointer ABI).
- VERIFICATION: feature_regression 549/549, integration 130/130,
  stdlib_tests 40/40, codegen lib 23/23; m249/m283/m326/m337/m371/m372/m377/
  m37_bug55 locks green; `p_wave196_small_gaps.xi` rc 0 on the released
  stdlib.
- RELAYS: `26bfcd9`. m376 acked->fixed with 9b6b3b59f (stdlib can re-add the
  mutating Map/Set combinations to `p_wave196_small_gaps.xi`). m377 is fixed
  (`d75f0de`) with the stdlib Vec4f lane-math follow-up
  `REL-20261020-1242-compiler` (use `data + index`, not `data + index * 4`).
- QUEUE: **m375 next** (orbitdb A6 memory: minimal `io.read_at` loop + RSS /
  `XIOM_TIMINGS` trace; do NOT change the arena/scoped-arena until measured).
  Then engine REL-20261019-2057, stdlib reds 1626/2343/2100-W1, m365;
  selfhost rebase + shapes; m373 profiling/m374; MCU 1220.
- CONSTRAINTS unchanged: rustc-first study; max 3-4 heavy sessions; watch
  memory; no stdlib edits except via relay; no release re-cut without owner
  GO; `--no-cache` for shared catidx/engine builds.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T15:55Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T15:55Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut or candidate cut without owner GO. Main `9b6b3b59f` has the
m376 mono-body param/ref reset (Map/Set helper AV) with lock
`e2e_m376_map_set_helpers`; `f5525e9f6` has the m377 scalar-pointee/call-site
pointer ABI fix with lock `e2e_m377_ptr_scalar_lane`. Relay `26bfcd9` marks
m376 fixed; m377's residual stdlib Vec4f lane-math item is
`REL-20261020-1242-compiler` (`data + index`, not `data + index * 4`). Earlier
main: C3 1f85922d8, m371 3b3e50323 + narrowing 095ac15a8, m372 0acbbdc8b,
m373/riscv 383fe9dc6, m374/MCU 154cc08d7. Selfhost branch 096bad953 (RD9 green;
rebase before the next harness).

Priority: 1) m375 next: minimal `io.read_at` loop + RSS/XIOM_TIMINGS trace on
v0.64.8; do not change the arena/scoped-arena until measured. Then engine
REL-20261019-2057, stdlib reds 1626/2343/2100-W1, m365. 2) Selfhost: rebase
096bad953, re-run T3, continue Option/.len, Vec.pop, &Vec+f64, index
arithmetic. 3) Performance: m373 profiling/XIOM_TIMINGS, m374 design, MCU
1220. 4) Wait for lane verification of the v0.64.8 check rows; no candidate
cut without owner GO.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere.

*** SESSION UPDATE 2026-10-20T12:47Z (compiler lane; m377 FIXED; stdlib lane-math follow-up) -- READ FIRST ***
- STATE: v0.64.8 remains the released archive; no re-cut and no candidate cut
  without owner GO.
- MAIN: `f5525e9f6` fix(codegen) m377 scalar-pointee inference + call-site
  pointer ABI (REL-20261019-2249-stdlib-2). Two roots: (1) the generic
  fallback bound T="*Float32" from an `as *Float32` cast target instead of
  Float32; (2) the call-site `*T` ABI substituted the stale
  `param_concrete_types` map, so it passed `i64*` to a `float*` mono def.
  Lock `e2e_m377_ptr_scalar_lane` (`tests/regression/m377_ptr_scalar_lane.xi`):
  warning-free compile, by-value lane round-trip `lane1=2 lane3=4`.
  Regression batch green: m372, m37 SIMD runtime, m77, m280, m312 (IR +
  runtime), codegen lib 23/23. Docs: COMPILER_BUGS m377 entry.
- RELAYS: `d75f0de`. m377 acked->fixed with the compiler commit; new
  `REL-20261020-1242-compiler` to stdlib: `Vec4f.get`/`set` still use
  `(data + index * 4)` on a `*Float32` field. `ptr + count` is ELEMENT-based
  (`api/ptr.md`), so the lane math double-scales and `Vec4f.get(1)` stays 0
  until stdlib uses `data + index` (or `ptr.offset` bytes). `Vec4f.new` is
  fine (its `data` is the i8* from `alloc`, byte-sized elements).
- QUEUE: **m376 next** (Map/Set generic-helper AV; add the combined mutating
  repro), then **m375** (minimal `io.read_at` loop + RSS/XIOM_TIMINGS trace;
  do not change the arena until measured). The m377 stdlib lane-math item
  blocks the reporter's `Vec4f.get` acceptance; compiler-side is green.
- CONSTRAINTS unchanged: rustc-first study; max 3-4 heavy sessions; watch
  memory; no stdlib edits except via relay; no release re-cut without owner
  GO; use `--no-cache` when shared catidx/engine builds interfere.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T12:47Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Read `docs/SESSION-HANDOFF.md` top block (2026-10-20T12:47Z),
`docs/RELEASE_GATE_v0.64.8.md`, and `release-checks/compiler-compiler-v0.64.8.md`
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std 0.64.8
green); no re-cut or candidate cut without owner GO. Main `f5525e9f6` adds the
m377 compiler fix (scalar-pointee generic inference + call-site pointer ABI)
with lock `e2e_m377_ptr_scalar_lane`; relay `d75f0de` marks m377 fixed and
opens `REL-20261020-1242-compiler` (stdlib Vec4f lane math must use
`data + index`, not `data + index * 4`). Earlier main: C3 absolute path
1f85922d8, m371 3b3e50323 + narrowing 095ac15a8, m372 0acbbdc8b, m373/riscv
383fe9dc6, m374/MCU 154cc08d7. Selfhost branch 096bad953 (RD9 green; rebase
before the next harness).

Priority: 1) m376 (add the combined Map/Set mutating repro, fix the AV), then
m375 (minimal read_at loop/RSS trace; no arena change until measured); then
engine REL-20261019-2057, stdlib reds 1626/2343/2100-W1, m365. 2) Selfhost:
rebase 096bad953, re-run T3, continue Option/.len, Vec.pop, &Vec+f64, index
arithmetic. 3) Performance: m373 profiling/XIOM_TIMINGS, m374 design, MCU
1220. 4) Wait for lane verification of the v0.64.8 check rows; no candidate
cut without owner GO.

Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine builds
interfere.

*** SESSION UPDATE 2026-10-20T12:26Z (compiler lane; next-session handoff) -- READ FIRST ***
- STATE: v0.64.8 released (tag `v0.64.8` @ `4d040604b`, 11 assets, run
  39296778685); `stdlib-v0.64.8` pinned; `xiom-std 0.64.8` registry green
  (run 39298965606, sha `c4ab5c61...`, OIDC). No re-cut and no candidate
  cut without owner GO.
- MAIN: `9e5904db2` (docs), with `1f85922d8` C3 absolute-path fix,
  `3b3e50323` m371 + `095ac15a8` narrowing, `0acbbdc8b` m372,
  `383fe9dc6` m373 config + riscv triple, `154cc08d7` m374/MCU checklists;
  README current at `f5b7b2f75`. Checker suite 213/213; m371/m372/m37 e2e
  locks green.
- RELAYS: `e619d29` (our triage `b4a77ab`). Newly acked: **m375** orbitdb
  A6 memory blow-up (57.8 GB WS at ~129k keys; per-I/O retention), **m376**
  stdlib Map/Set generic-helper AV, **m377** Vec4f `*Float32` lane
  miscompile; benchmark matrix acked as m373 baseline (front-end-bound,
  ~85 MB RSS floor, LSI 45).
- SELFHOST: branch `096bad953` (RD9 green: T3 6/6, T6 14/64; base
  `154cc08d7`; not pushed) -- rebase before the next harness.
- OPEN CHECK: v0.64.8 reporter rows (engine soak, xvector, orbitdb,
  bindings, PULSE; v0.64.7 rows) + candidate decision after verification.
- NEXT-SESSION PRIORITY: m377 -> m376 -> m375; engine REL-20261019-2057;
  stdlib reds 1626/2343/2100-W1; m365; selfhost rebase + shapes; m373
  profiling/m374; MCU 1220; m363/m366.
- CONSTRAINTS: rustc-first study; max 3-4 heavy sessions; watch memory; no
  stdlib edits except via relay; no release re-cut without owner GO; use
  `--no-cache` when shared catidx/engine builds interfere.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-20T12:26Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Start by reading docs/SESSION-HANDOFF.md top blocks (2026-10-20T12:26Z),
docs/RELEASE_GATE_v0.64.8.md, and release-checks/compiler-compiler-v0.64.8.md
in E:\xiom-lang\xiom-relays.

State: v0.64.8 is released (tag 4d040604b, 11 assets, registry xiom-std
0.64.8 green); no re-cut or candidate cut without owner GO. Main 9e5904db2
has the post-release fixes: C3 absolute path 1f85922d8, m371 3b3e50323 +
narrowing 095ac15a8, m372 0acbbdc8b, m373 config/riscv 383fe9dc6, m374/MCU
checklists 154cc08d7; README f5b7b2f75. Selfhost branch 096bad953 (RD9
green: T3 6/6, T6 14/64; rebase before the next harness).

Relays just triaged: m375 orbitdb A6 memory blow-up (57.8 GB WS at ~129k
keys; probe_soak.xi; per-I/O retention in read_at/u8_to_bytes or
block_store_open), m376 stdlib Map/Set generic-helper AV
(p_wave196_small_gaps.xi), m377 Vec4f *Float32 lane miscompile
(llvm_type_for defaults to i64). The benchmark matrix is the m373 baseline
(front-end-bound, ~85 MB RSS floor, LSI 45; benchmark commit 2b0051c).

Priority:
1) Bugs: m377 first (small pointer-to-scalar mapping), then m376 (add the
   combined Map/Set repro, fix the AV), then m375 (minimal read_at loop/RSS
   trace; do not change the arena until measured). Then engine
   REL-20261019-2057, stdlib reds 1626/2343/2100-W1, m365.
2) Selfhost: rebase 096bad953 onto current main, re-run T3, continue
   Option/.len, Vec.pop, &Vec+f64, index arithmetic.
3) Performance: m373 profiling using the benchmark matrix and XIOM_TIMINGS
   on engine/xvector/orbitdb; m374 design; MCU 1220 backend.
4) Wait for lane verification of the v0.64.8 check rows; no candidate cut
   without owner GO.
Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; use --no-cache when shared catidx/engine
builds interfere.

*** SESSION UPDATE 2026-10-20T12:20Z (compiler lane; incoming relays triaged) -- READ FIRST ***
- RELEASE: v0.64.8 remains the released archive (tag 4d040604b, 11 assets,
  registry xiom-std 0.64.8 green). No re-cut; no candidate cut without
  owner GO.
- INCOMING (acked 2026-10-20; relay b4a77ab): **m375** orbitdb A6 1M-key
  soak memory blow-up (57.8 GB WS at ~129k keys; per-I/O retention in
  io.read_at/u8_to_bytes or block_store_open buffers); **m376** stdlib
  generic Map/Set helper codegen AV (mutating wrapper combo); **m377**
  Vec4f `*Float32` lane miscompile (llvm_type_for defaults to i64); the
  v0.64.8 benchmark matrix is acked as the m373 baseline (front-end-bound,
  ~85 MB peak RSS, LSI 45).
- EXISTING QUEUE: engine REL-20261019-2057 (fn-type alias param + global
  Vec.pop store-back); stdlib reds 1626/2343/2100-W1; m365 dep-cache;
  m363 Send/Sync; m366 blocked on the engine snippet; MCU 1220 backend;
  m373 profiling/m374 design; selfhost rebase + Option/.len, Vec.pop,
  &Vec+f64, index arithmetic.
- README updated to v0.64.8 status (`f5b7b2f75`): current release, registry,
  post-release fixes, selfhost RD9 14/64, RISC-V triple, registry URL.
- v0.64.8 check rows remain reporter-side (engine soak, xvector, orbitdb,
  bindings, PULSE; v0.64.7 rows). No open compiler release thread.

*** SESSION UPDATE 2026-10-19T17:50Z (compiler lane; v0.64.8 RELEASED) -- READ FIRST ***
- STATE: **v0.64.8 is RELEASED.** Annotated tag `v0.64.8`
  (`b0261db4d39f532a4f2cd697faed271c0228fe15`) on `4d040604b`; tag push
  release run **39296778685 SUCCESS** (guard + windows/linux/macOS/VSIX
  package legs + publish; docs dispatch green). GitHub release live
  https://github.com/xiom-lang/xiom/releases/tag/v0.64.8 (2026-10-19T17:49Z)
  with 11 assets. `STDLIB_VERSION=stdlib-v0.64.8` at the tag; stdlib r2
  gate run **39289746856 SUCCESS** (`smoke_io_edge` fixed).
- ASSET SHAS (sha256): windows-x64.zip
  `c31d9c464a4f9489fc442b4bf9a45b3e3b2363dcf9548d53566bd04a81dee6b5`;
  linux-x64.tar.gz
  `a8c003c0d817880a8754fa301a9ba18359cdc7463d7cfc98d3267d7a2e56002a`;
  linux-arm64
  `e773260f8ef18998de782a8fe6d8993b0ad92c62c7e406be20d65e708bc3fab6`;
  SHA256SUMS
  `f390844ff2f9b5eb524a1a41b97037862c1e1a7ffeedc127856f87715cbd11ad`.
  The Windows archive was verified locally (hash + `bin/` + `lib/` staged;
  stdlib imports work without `XIOM_STDLIB`).
- REGISTRY: **xiom-std 0.64.8 PUBLISHED.** publish-registry run
  **39298965606** (tag ref `stdlib-v0.64.8`, `compiler=v0.64.8`) is green
  after ops restored the OIDC trusted-publisher mapping (the registry
  2.14.2 recreate had dropped it): registry sha256
  `c4ab5c614506c44f95c011f81901d1630f7d4788aa51af6d5b86ca013447f4fe`,
  publisher `xiom-lang/stdlib` with OIDC provenance, registry latest =
  0.64.8. The earlier `publisher_not_mapped` runs are superseded. No open
  release thread; reporters fill the check rows.
- POST-v0.64.8 FIX: main `1f85922d8` fixes absolute-path C3 package
  discovery (m369's shallow temp index skipped `packages/`); the real
  playground tree passes `--check` absolute and relative, catalog locks
  2/2. No v0.64.8 re-cut; the next cut carries it. Engine j-spine
  `REL-20261019-2057` acked; stdlib gate-9 green items 1053/1456/2328/2000
  moved to fixed, reds 1626/2343/2100-W1 queued.
- POST-v0.64.8 FIX 2: m371 imported-struct alias layout (`3b3e50323`,
  narrowed `095ac15a8` to restore `m37_bug55`) and m372 nested `ptr.read`
  inference (`0acbbdc8b`). Locks: e2e_m371 + e2e_m372 + e2e_m37 green;
  checker suite 213/213 incl `catalog_corpus_is_clean`. No re-cut.
- POST-RELEASE VERIFY (reporters fill the v0.64.8 check rows): engine
  2032/2053/1343 soak on the archive; xvector XVC-C-20 + G-2b; orbitdb
  natural position tuple 160/160 + C-ORBIT-13; bindings B-34 (B-29
  not-a-defect, B-32 retired); PULSE 1309/2301 can re-enable Phase B on a
  main build or the next cut (m371 fixed on main). v0.64.7 is not re-cut.
- NEXT WAVE (owner agreed): **m373** parallel compile knobs + product
  profiling (`xiom.toml [compiler]` parallel/jobs/parallel-codegen; use the
  existing 7C/I2 paths; XIOM_TIMINGS on engine/xvector/orbitdb; parallel C
  deps via m365); **m374** parallel checker/codegen units (design/queue
  now, implement after selfhost Phase 6 parity); MCU/bare-metal 1220/1419.
- RELAYS: m365 remainder; m366 blocked on the engine snippet; m363
  Send/Sync; bindings B-29/B-34/B-32 closure; xvector XVC-C-20/G-2b;
  orbitdb C-ORBIT-12/13; engine soak; .xiomignore/alias/LLVM/packages
  re-verify.
- SELFHOST: RD9 GREEN on `154cc08d7` (RD9 commit `096bad953`, not pushed);
  main has since advanced through m371/m372 (`095ac15a8`) -- rebase before
  the next harness. Ported the v0.64.8 guard alloc/realloc drift
  (`@malloc`/`@realloc` -> `@xiom_guard_alloc`/`@xiom_guard_realloc` + zero-cap
  select); full T3 suite 6/6 (942.71 s), meter 19/19 + 14/64. Next:
  Option/.len, Vec.pop, &Vec+f64, index arithmetic.
- CONSTRAINTS unchanged: rustc-first study; max 3-4 heavy sessions; watch
  memory; no stdlib edits except via relay; no release re-cut without
  owner GO; use `--no-cache` when shared catidx/engine builds interfere.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-19T17:50Z ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Start by reading docs/SESSION-HANDOFF.md top blocks (2026-10-19T17:50Z),
docs/RELEASE_GATE_v0.64.8.md, and release-checks/compiler-compiler-v0.64.8.md
in E:\xiom-lang\xiom-relays.

v0.64.8 is RELEASED (tag v0.64.8 @ 4d040604b, run 39296778685, 11 assets,
stdlib-v0.64.8 pin). The release is fully closed: compiler v0.64.8,
stdlib-v0.64.8, and xiom-std 0.64.8 are public; reporters fill the check
rows as they verify on the archive.

Next wave (owner agreed): m373 parallel compile knobs + product profiling --
expose [compiler] parallel/jobs/parallel-codegen in xiom.toml, use the
existing 7C/I2 paths, profile engine/xvector/orbitdb with XIOM_TIMINGS,
parallel C deps via m365; m374 parallel checker/codegen units -- queue/design
now, implement after selfhost Phase 6 parity; MCU targets 1220/1419.
Relays: m371 PULSE imported-struct layout, m365 remainder, m366 blocked on
the engine snippet, m363 Send/Sync, m372 nested ptr.read, bindings
B-29/B-34/B-32 closure, xvector XVC-C-20/G-2b, orbitdb C-ORBIT-12/13, engine
2032/2053/1343 soak, .xiomignore/alias/LLVM/packages re-verify.
Selfhost: RD7 green T6 14/64; continue Option/.len, Vec.pop, &Vec+f64, index
arithmetic; rebase before the next harness.
Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; no release cuts without owner GO; use
--no-cache when shared catidx/engine builds interfere.

*** SESSION UPDATE 2026-10-19T17:40Z (compiler lane; v0.64.8 TRIGGER PENDING -- r2; SUPERSEDED -- v0.64.8 released, see 17:50Z block above) ***
- STATE: **v0.64.8 is NOT tagged/released yet**; v0.64.7 is not re-cut.
  Main `b661f645f`; fix commit `4d040604b`; release gate
  `docs/RELEASE_GATE_v0.64.8.md`; v0.64.8 release check in the relay repo.
- R2 GATE: the r1 stdlib gate failed `smoke_io_edge` (SIGILL /
  0xC000001D) because empty-file `Vec[UInt8]::with_capacity(0)` hit
  `xiom_guard_alloc(0)`=NULL -> `vec_wc_trap`; fixed in r2 `4d040604b`
  (`emit_alloc` clamps zero to 1; Vec.push grows from 8).
  `candidate-v0.64.8-r2` pushed; stdlib tag moved to `112c5b6`
  (`COMPILER_VERSION=candidate-v0.64.8-r2`). r2 dry-run `39287949178`
  **SUCCESS**; r2 stdlib run `39289746856`: Ubuntu gate **SUCCESS**, Windows
  **full smoke SUCCESS**, final Windows checks running. Combined notes
  re-verified against the moved tag.
  **Next session:** if `39289746856` completes green, tag `v0.64.8` at
  `4d040604b`, push, watch the release, send the final tag/asset SHA to
  stdlib for the registry override, and announce/unblock the lanes. If red,
  do not tag; report. No v0.64.7 re-cut.
- NEXT-WAVE DECISIONS (owner 2026-10-19, agreed):
  - **m373 -- parallel compile knobs + product profiling** (next wave):
    expose `parallel`, `jobs`, `parallel-codegen` (and optional
    `codegen-units`) in `xiom.toml [compiler]`, map to `CompileConfig`; use
    the existing 7C (parallel lex/parse across files) and I2 (per-function
    codegen) paths; profile engine/xvector/orbitdb with `XIOM_TIMINGS`;
    parallel C deps are m365. Keep selfhost parity defaults sequential.
  - **m374 -- parallel checker/codegen units** (design/queue now;
    implementation after selfhost Phase 6 parity, mirror in selfhost):
    module-DAG checking, CGUs, deterministic diagnostics/IR. Not in v0.64.8.
  - **MCU/bare-metal target** (1220/1419): `--target arm` is AArch64 Linux,
    riscv triple invalid; Cortex-M/thumb/no_std/QEMU.
  - **All relays + v0.64.8 feedback/bugs**: m371 PULSE imported-struct
    layout, m365 remainder, m366 blocked on the engine snippet, m363
    Send/Sync, m372 nested generic ptr.read; bindings B-29/B-34/B-32
    closure; xvector XVC-C-20/G-2b; orbitdb C-ORBIT-12/13; engine
    2032/2053/1343 soak; .xiomignore/alias/LLVM/packages re-verify.
- TOML/PARALLEL FACTS (for m373): `xiom.toml` is found by walking up from
  the entry file; `[project]` name/version/root/source-roots;
  `[dependencies]` dotted registry specs; `[compiler]` currently parses
  target/release/incremental/max-depth/timeout-secs -- **no parallel/jobs/
  parallel-codegen yet**. `package.xi` is the registry manifest; `xiom.toml`
  is local build resolution. 7C already parallelizes lex+parse across files
  (`--parallel`, `--jobs`); I2 already parallelizes function bodies
  (`--parallel-codegen`); the checker and module/CGU partitioning are still
  sequential.
- SELFHOST: RD7 green (6/6, 418 s, T3 19/19, T6 14/64, header 8586); next
  shapes Option/`.len()`, `Vec.pop`, `&Vec`+f64, index arithmetic; branch
  `selfhost-phase-4-codegen` 47 ahead / 0 behind before the next wave.
- CONSTRAINTS unchanged: rustc-first study; max 3-4 heavy sessions; watch
  host memory; no stdlib edits except via relay; no release re-cut without
  owner GO; use `--no-cache` when shared catidx/engine builds interfere.

*** CONTINUATION PROMPT (paste-ready) -- 2026-10-19T17:40Z (SUPERSEDED -- use the 17:50Z prompt above) ***
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom).
Start by reading docs/SESSION-HANDOFF.md top blocks (2026-10-19T17:40Z),
docs/RELEASE_GATE_v0.64.8.md, and release-checks/compiler-compiler-v0.64.8.md
in E:\xiom-lang\xiom-relays.

v0.64.8 is NOT released yet. The r1 stdlib gate caught a SIGILL in
smoke_io_edge (zero-capacity Vec); it is fixed in r2 4d040604b /
candidate-v0.64.8-r2. The stdlib tag is moved to 112c5b6; the r2 stdlib run
39289746856 has Ubuntu and Windows full-smoke green (final Windows checks
running); r2 dry-run 39287949178 SUCCESS; notes verified with the moved tag.

1) Finish the release: check 39289746856. If green, tag v0.64.8 at
   4d040604b, push, watch the release workflow, send the final tag/asset SHA
   to stdlib for the registry override, and announce/unblock
   engine/xvector/orbitdb/bindings/PULSE. If red, do not tag; report. No
   v0.64.7 re-cut.
2) Next wave (owner agreed): m373 parallel compile knobs + product profiling
   -- expose [compiler] parallel/jobs/parallel-codegen in xiom.toml, use the
   existing 7C/I2 paths, profile engine/xvector/orbitdb with XIOM_TIMINGS,
   parallel C deps via m365; m374 parallel checker/codegen units -- queue/
   design now, implement after selfhost Phase 6 parity; MCU targets 1220/1419.
3) Relays: m371 PULSE imported-struct layout, m365 remainder, m366 blocked
   on the engine snippet, m363 Send/Sync, m372 nested ptr.read, bindings
   B-29/B-34/B-32 closure, xvector XVC-C-20/G-2b, orbitdb C-ORBIT-12/13,
   engine 2032/2053/1343 soak, .xiomignore/alias/LLVM/packages re-verify.
4) Selfhost: RD7 green T6 14/64; continue Option/.len, Vec.pop, &Vec+f64,
   index arithmetic; rebase before the next harness.
Constraints: rustc-first study; max 3-4 heavy sessions; watch memory; no
stdlib edits except via relay; no release cuts without owner GO; use
--no-cache when shared catidx/engine builds interfere.

*** SESSION UPDATE 2026-10-19T14:50Z (compiler lane; v0.64.8 RELEASE IN PREP -- OWNER GO) -- READ FIRST ***
- **v0.64.8 r1 STDLIB GATE RED; r2 FIX IN MAIN `4d040604b`
  (2026-10-19T16:35Z)**: stdlib release gate on candidate r1 (`c45a247d2`)
  failed `smoke_io_edge` on both OSes (compile rc 0, SIGILL 0xC000001D):
  empty-file `Vec[UInt8]::with_capacity(0)` -> `xiom_guard_alloc(0)` NULL ->
  `vec_wc_trap` -> `llvm.trap`; push from cap 0 also `realloc(ptr,0)` ->
  NULL. Fixed: `emit_alloc` clamps zero-size requests to 1 (positive
  constants keep the literal IR shape; runtime sizes use a select), and
  `Vec.push` grows from first capacity 8 when cap == 0. Debug
  `smoke_io_edge` rc 0; new fixture/e2e `vec_with_capacity_zero` 1/1;
  m296/m314/m195/vec locks green; m37 50/50; v0.64.7 control rc 0.
  Candidate **`candidate-v0.64.8-r2`** pushed at `4d040604b`; stdlib moved
  `stdlib-v0.64.8` to **`112c5b6`** (`COMPILER_VERSION=candidate-v0.64.8-r2`)
  and re-ran (`REL-20261019-1624-stdlib`). r2 dry-run **39287949178
  SUCCESS** on `4d040604b` (all package jobs); r2 stdlib run 39289746856:
  Ubuntu gate SUCCESS, Windows **full smoke corpus SUCCESS**
  (`smoke_io_edge` fixed), remaining Windows checks running. Combined notes
  were re-verified against the moved tag (6 highlights). **No v0.64.8 tag
  until the stdlib run completes green.**
- STATE: **owner GO for the next cut, v0.64.8**, built from current main
  (fix baseline `ac20559f9`; release prep `c45a247d2`). No v0.64.7 re-cut.
  Main is `4d040604b`; relay bus `229f87b`. Release gate:
  `docs/RELEASE_GATE_v0.64.8.md`.
- PREP DONE: workspace + selfhost version bump to 0.64.8, Cargo.lock, VSIX
  README pin (`c45a247d2`); release notes `release-notes/v0.64.8.md` plus
  the **combined 6-highlight JSON** (4 compiler + 2 stdlib from candidate
  `9b88b46`, verify green; `37f0ba2c1`); v0.64.8 release check opened
  (`release-checks/compiler-compiler-v0.64.8.md`).
- RELEASE PREP UPDATE (15:15Z): the compiler source ref for the stdlib build
  is **`candidate-v0.64.8-r1` / `c45a247d2`** (workspace 0.64.8; pushed).
  Do NOT use `ac20559f9` -- it is pre-version-bump 0.64.7. The stdlib lane
  had not tagged as of this update. Battery1 was not a code regression: 5
  suites failed on (a) Vec-allocation locks still expecting `@malloc` (now
  guard-alloc), (b) the release-layout runtime C missing, and (c) the
  stdlib root for API-freeze/JIT. Fixed the Vec allocation locks
  (`087e5e7f1`), staged the `9b88b46` stdlib into `target/release/lib`,
  rebuilt release 0.64.8. The engine editor build finished, so the release
  binary is free to relink.
- **BATTERY2 GREEN ON THE COMPILER SIDE**: e2e **2612/2613 + m17 clean
  re-run** (the single failure was a local duplicate stdlib root: repo
  `stdlib` + staged `target/release/lib` -> W001; m17 passes when the
  staged lib is removed); feature_regression 549/549, integration 130/130,
  stdlib_execution 85/85, stdlib_tests 40/40, api_freeze 3/3, rtcache/JIT
  72/72, checker 212/212, codegen lib 23/23. Log:
  `%TEMP%\kilo\v0648-battery2.log`.
- **DRY-RUN 39276425781 FAILED at "Checkout stdlib at the pin"**: the
  stdlib candidate `9b88b46` is **LOCAL-ONLY** (not on origin). This is not
  a compiler failure. `STDLIB_VERSION` was reverted to `stdlib-v0.64.7`
  (`6faf74dfb`); relay `REL-20261019-1532-compiler` asks stdlib to push
  `9b88b46` and tag `stdlib-v0.64.8`. Re-run the dry-run with the tag once
  it is on origin.
- HARD GATE: **stdlib-v0.64.8 must be tagged and published before the
  compiler tag**; the stdlib request is `REL-20261019-1432-compiler`
  (tag/publish + `release-notes/v0.64.8.md` fragment). The **source ref is
  provided** (15:13Z): `candidate-v0.64.8-r1` / `c45a247d2`; the stdlib
  tag/publish was still pending at this update. Sequence after it lands: `STDLIB_VERSION=stdlib-v0.64.8` pin -> convert/verify release notes
  with the pinned fragment -> candidate dry-run archive -> full battery ->
  tag `v0.64.8` on main -> release + docs dispatch.
- THIS CUT includes m360/m368/arena/m369/m370/m367/C-ORBIT-12/13/XVC-C-20/
  B-34 and is the archive requested by the engine (`REL-20261019-1312`).
  NOT in this cut: m371 PULSE imported-struct layout (1309/2301), m372
  nested generic ptr.read (1451; workaround in the channel handle form), m366
  capture-frames hang (blocked on the engine snippet), m365 caller flag
  split/per-key lock, m363 Send/Sync, MCU/bare-metal, selfhost Phase 6
  (separate track). These are known issues/next cycle.
- ENVIRONMENT: the engine editor build **finished**; `target/release/xiom.exe`
  was rebuilt as 0.64.8 and `target/release/lib` was staged with the
  `9b88b46` stdlib for the local battery. The local battery uses the debug
  0.64.8 compiler plus that staged stdlib; CI builds the archive fresh.
- v0.64.7 check remains open with 114 unverdict rows (reporters own
  fixed->verified); the v0.64.8 check owns the new cut. Full battery plan:
  e2e full + feature_regression + integration + stdlib_execution +
  stdlib_tests + api_freeze + checker, after the engine build exits.

*** SESSION UPDATE 2026-10-18T16:15Z (compiler lane; v0.64.7 UNCHANGED -- RETESTS + FIXERS) -- READ FIRST ***
- STATE: v0.64.7 remains the release (no re-cut). Main `dba86f6a8`;
  relay bus updated by the compiler lane (see the m360/m368/m369 items).
- BENCHMARK CHECK (REL-20261018-0042-benchmark): acked, NO ACTION. It
  confirms v0.64.7 absorption (commit 79993a1, image sha256:21b4ad95...),
  acceptance 42/42 + 36/36 + 16/16 + lz4 3/3, and the m252 cold lanes are
  cold from sample 1 (aot t6 1576-1686 ms / t7 5874-6028; jit t6 1572-1885 /
  t7 5824-6055; warm-up == median after the one-time 5.5 s JIT rtobj build).
  The t3 trampoline row stays acked awaiting a compiler profile.
- VEC-DROP **FIXED + ON MAIN `5a530cf7e`** (docs `431e11f23`;
  2026-10-18T18:30Z): the engine repro (3000 x 1M Float64 push/drop)
  **completes rc 0 in 19.2 s** with a flat **15.4 MB WS / 47.5 MB private**
  peak on the rebased release build (v0.64.7 baseline: 10.12 GB WS / 10.62
  GB private at t=10 s, still growing; v0.64.4 15.9/16.7 GB). Codegen frees
  owned/non-escaping Vec locals at scope exit (conservative AST prepass +
  entry-block drop flags, `@xiom_guard_free`); escaped/aliased Vecs stay
  leak-only (no UAF). Branch `fix/m368-vec-drop-reclaim` was rebased onto
  main and integrated; locks e2e_m368 1/1, m370 1/1, m37 sweep 50/50, fixer
  battery green. Relay 2032 moved to **fixed**; the engine can retire the
  workaround. No re-cut.
- M360 RELAUNCHED (REL-20261016-2331-stdlib): v0.64.7 baseline CONFIRMED
  RED -- build rc 0, direct run rc -1073741819 (0xC0000005) 3/3 on the
  official archive; expected rc 0/value 2. Branch
  `fix/m360-i128-div-vec-loop-av`, session `ses_eb0420efdffehWu33lcttC01ED`.
  Harness pitfall: Start-Process can report a blank/0 ExitCode for the AV;
  check the raw status directly.
- **m360 RESOLVED + ON MAIN (2026-10-18T19:55Z)**: root cause was the stdlib
  runtime helper `xiom_udivti3`'s single-limb fast path (`a / b` lowers to
  `__udivti3`, the symbol that TU defines -> self-recursion), NOT the vector
  loop/namespace lowering; the loop only blocked constant folding. Compiler
  fix: i128 `/`/`%` now lower to a 128-step inline long division (no
  sdiv/srem/udiv/urem, no runtime helper), with INT128_MIN/-1 wrap and the
  div-by-zero trap retained. Fix `728bf579a` (branch pushed) cherry-picked to
  main `dc30929da` after main-lane witness verification (compile rc 0 + run
  rc 0 x2 at default and O0, value 2; official archive still AVs). Locks:
  m360 e2e 1/1, m314 2/2, m37_u128 1/1, m37_index_arith 1/1, m37 sweep
  50/50, codegen lib 23/23. Relay REL-20261016-2331-stdlib moved to
  **fixed**; stdlib residual = replace the runtime helper's single-limb
  branch for direct C callers (none remain in XIOM codegen).
- **m370 FIXED + ON MAIN (2026-10-18T20:10Z)**: new stdlib HIGH item
  REL-20261018-1546 (`&Vec[GenericStruct]` callee field corruption; breaks
  async_select closed-drain). Reproduced on v0.64.7: compile rc 0, run
  `plain=111` rc 1. IR showed `bs[0]` taking the scalar i64 elem_load and
  the field reads becoming constants; root cause = the `Box[Int]` annotation
  spelling vs the erased `{module}.Box` registry key. Fix
  `resolve_vec_elem_type` generic-base fallback (`f7fe288e3`, branch
  `fix/m370-vec-byref-generic`, pushed); witness now `plain=0` rc 0; locks
  m370 e2e 1/1, m37 sweep 51/51, m218 1/1. Relay moved to **fixed**;
  stdlib to re-test async_select on a main build (v0.64.7 still affected).
- NEW XVECTOR ITEM (REL-20261018-1518-xvector, high): triaged as NOT a
  global-initializer hang. The exact probe is green on the same archive
  (build.ps1 + direct; "main started"/"vec global ok", exit 0); even the
  safe null-only mutex control stalls 43-56 s, then completes. ROOT CAUSE:
  `$HOME/.xiom/catidx.txt` (11.9 MB, 52 stale stdlib roots incl.
  `tests/regression` fixtures) is shared across compiler versions; a
  concurrent v0.64.4 engine build flipped the header 0.64.7 <-> 0.64.4 and
  each flip forces a full rebuild/scan (exact probe 7.9 s when free; 6-line
  file 49-56 s during a flip). `XIOM_TIMINGS=1` phase split: index 50.6 s /
  56,663 misses vs check 0.7 s / codegen 0.03 s -- scan BREADTH (source-dir
  shape) is the cost, even under `--no-cache`; scope the scan to declared
  source roots + stdlib, never walk CWD/temp trees. Reply posted on the
  item. **m369 FIXED + ON MAIN `330955136` (2026-10-18T18:05Z)**: transient
  temp file-parents are shallow source dirs (no recursion, no grandparent),
  the cache is per compiler version/platform
  (`catidx-<version>-<os>-<arch>.txt`), and saves prune entries outside the
  current source roots. Measured: 6-line temp-tree compile index
  **50.569 s -> 0.707 s** (`--no-cache`) / 0.392 s warm; 56,663 -> 2,058
  stdlib-scope misses. Locks: catalog 5/5, `xiom-check` 211/211, m347 e2e
  3/3. The xvector item moved to fixed; re-test lane workloads on a main
  build (the v0.64.7 archive retains the old scan behavior). Plan/history:
  `docs/checklists/m369-catalog-index.md`.
- SELFHOST RD6 **GREEN** (2026-10-18T20:09Z): rebase onto main `3ba31eb7f`
  (m360/m370) + RD6 committed as `a2a0ebad7`; full suite **6/6 in 796.47 s**,
  T3 19/19, T6 13/64, IR headers 8503 (258/4/8075), check 83 files + 75
  manifest cases; ascii OK. `m37_tuple_struct` adoption re-attempted and
  **BLOCKED**: div_mod + triple are IDENTICAL, but `mk_big` stubs on
  `x.d.push(v)` (FIELD receiver -> **P5e**) and `main` stubs on tuple-call
  local reads + field chains over call results (**P2b-iv**); T6 stays 13/64.
  Next selfhost work = P5e + P2b-iv, then m37 (14/64). Branch is 45 ahead /
  6 behind main (the m369 catalog change is reference-only; rebase before
  the next full harness).
- SELFHOST P5e + P2b-iv + m37 **GREEN** (2026-10-18T21:08Z): commit
  `3b5b0f8fd` (unpushed; 46 ahead / 6 behind main). Field-receiver
  `x.d.push(v)` (P5e) and expression-receiver Vec index (P2b-iv) ported;
  `m37_tuple_struct` FULL FILE IDENTICAL (747 lines) and adopted into T6.
  Suite **6/6 in 594.86 s**; meter 19/19 + **T6 14/64**; header parity
  8586 (258/4/8158 = 8503 + 83 `guard_free` declares); ascii OK. The
  selfhost session owns the next Phase 6 shapes; rebase before the next
  full harness.
- SELFHOST RD7 **GREEN** (2026-10-18T21:52Z): rebase onto main `b3496f360`
  (XVC-C-20/C-ORBIT-12/m365) committed as `5c41c389a`; suite **6/6 in
  418.06 s**; meter 19/19 + **T6 14/64**; header parity 8586 unchanged;
  ascii OK; 47 ahead / 0 behind main. Next shape costs noted: Option
  payloads/`.len()`, the `Vec.pop` family, `&Vec` receivers + f64 elements,
  index arithmetic.
- **m367 FIXED + ON MAIN `90244d42c` (2026-10-18T18:45Z)**: project-root
  catalog scans skip `tests/regression`/`tests/fixtures` trees; a compile
  whose source root IS a fixture still indexes its own files and `mini_std`.
  The m347 `mini_std` no longer leaks W001 into root `--dump-check` or
  suppresses `CHECK-OK`. Catalog 6/6, m347 e2e 3/3. No re-cut.
- **C-ORBIT-12 RESIDUAL FIXED + ON MAIN `c6b3362ef` (2026-10-18T19:10Z)**:
  the residual was the applied generic alias in an IMPORTED signature
  (`DbResult[(TValue, Int)]`), not the enum/tuple namers. `llvm_type_for`
  now registers an unresolved `Tuple__...` on demand under the spelling the
  access expects (each segment's registered type, bare key preferred). New
  lock `e2e_m336_alias_module_tuple`; m336 2/2, m324 1/1, m37 50/50. The
  natural position tuple is restored on orbitdb branch `f779c23` (scratch
  worktree `E:\xiom-projects\xiom-orbitdb-m336`): **XIOM DB Conformance
  160/160** on the release compiler (also 160/160 debug). Orbitdb to verify
  and retire the workaround. Relays 1154 + 1416 moved to fixed; the
  extraction request 1650 closed.
- **XVC-C-20 FIXED + ON MAIN `56d362a31` (2026-10-18T22:10Z)**: `&mut` on a module-level
  aggregate LOADED a copy and passed a temporary; the global never saw the
  mutation (`count after create = 0`, second create Ok). The `&x` Ident path
  now resolves `module_globals`: aggregate globals return `@symbol`, scalars
  the slot address, pointer globals the stored pointer (R43). Minimal
  fixture + `e2e_xvc_c20_global_aggregate` 1/1; the real xvector repro now
  `count=1`, second create Err, rc 0 on release + debug; m37 50/50, m337
  3/3. Relay 1815 fixed; xvector to verify (the second-create error message
  prints a numeric handle -- separate from the aliasing fix).
- **B-34 FIXED + ON MAIN `344abbc23` (2026-10-18T22:50Z)**: a cross-module alias in a
  generic position (`&Vec[Handle]`, `Handle = Int`) now unifies with
  `Vec[Int]`; the generic-argument helpers resolve aliases for both the
  argument and the container base before comparing. Fixture
  `b34_vec_alias_generic` runs `n=1 h=42` on debug + release; new lock
  `e2e_b34_vec_alias_generic` 1/1; `xiom-check` 212/212; m336 3/3; m37
  sweep 50/50. Relay 0042 fixed; B-29 is the only reporter-side re-run left
  in this cluster (B-32 retired).
- **C-ORBIT-13 FIXED + ON MAIN (2026-10-19T14:00Z)**: `&mut` on a
  >=3-level nested field path (`&mut t.store.bs`) now follows the real
  lvalue chain with `lvalue_ptr` GEPs instead of loading intermediate
  copies. The fixture prints two-level/three-level/workaround PASS, rc 0 on
  debug + release (pre-fix three-level `n=1 g=0 v=1`); new lock
  `e2e_c_orbit_13_nested_mut_path` 1/1; m37 50/50, m336 3/3, XVC-C-20 1/1,
  B-34 1/1. Relay 1209-orbitdb fixed; orbitdb can retire the pager
  workaround after re-verify.
- **ESCAPED/PER-FRAME VEC LEAK FIXED (REL-20261019-1343-engine)
  (2026-10-19T15:00Z)**: all compiler-emitted allocations now route through
  the guard-aware allocator/realloc (main-heap fallback when no arena is
  active); `xiom_guard_heap_enter()` / `xiom_guard_heap_exit()` are
  explicitly SAFE externs (checker allowlist), giving a documented scoped
  arena with no `unsafe` in application code. Editor-shaped witness:
  **4555 MB WS / 4765 MB private -> 51.7-53.3 MB WS / 82.7 MB private**
  (flat, 8.5-9.9 s); m368 loop still flat 15.0 / 47.5 MB; new lock
  `e2e_escaped_frame_arena` 1/1; m37 50/50, m336 3/3, C-ORBIT-13 1/1, B-34
  1/1, XVC-C-20 1/1, m347 3/3, checker green. Engine must wrap each editor
  frame (keep persistent state on the main heap) and run the capped 600-frame
  soak. Docs: `docs/SCOPED-ARENA.md`; soak: `tests/soak/`. Archive 1312
  still needs owner GO + the remaining lane verification.
- **PULSE 1309 TRIAGED / m371 OPEN (2026-10-19T15:30Z)**: reproduced on
  v0.64.7 AND current main with a two-module `type HandlerOut =
  app.HandlerOut;` probe (T001 field errors). The checker can resolve the
  alias fields, but the codegen registry has no imported `HandlerOut` layout
  (`[alias-debug] name=app.HandlerOut known=false keys=[]`), emits
  `out_json`/`local_out` as i64, and the runtime reads `s=0 n=4294967295
  ct=0`. The checker-only fix was REVERTED (it converts a safe compile error
  into silent corruption). m371 queued: register imported-module struct
  layouts in codegen, then re-enable field access and verify
  `app.pre_stop(co)`. PULSE keeps the loop workaround until then.
- QUEUE (triaged 2026-10-19): **in flight** none (m360, m370, m368, m367,
  C-ORBIT-12, XVC-C-20, B-34, C-ORBIT-13 and the escaped/per-frame Vec leak
  are fixed on main; selfhost RD7 green, T6 14/64; next Phase 6 shapes).
  **IMMEDIATE:** PULSE 1309 reproduced; **m371 (codegen imported-struct
  layout) queued**; the loop workaround stays until it is green.
  **Next wave** (do not launch until a heavy slot frees): (a) m365 PARTIAL:
  ready marker + canonical dependency paths landed; remaining caller flag
  split + per-key lock and the engine re-measure (checklist
  `docs/checklists/m365-dep-cache.md`); (b) m363 Send/Sync enforcement
  (THREADING_PLAN Phase B: the spawn-capture `is_send` half exists, but
  `is_send` assumes every generic container is Send, `is_sync` does not
  exist, and Channel/Mutex/Arc boundaries are unchecked; checklist
  `docs/checklists/m363-send-sync.md`); (c) m372 nested generic `ptr.read`
  (`REL-20261019-1451-stdlib`; silent erased-T read; the channel handle form
  has a stage-in-local workaround); (d) m366 **BLOCKED** on the engine's
  minimized capture-only snippet (asked 2026-10-17); (e) .xiomignore
  1504/2104; alias-qualified 2100; LLVM baseline 1647; packages 2303/2309
  re-verify (registry canary 2257 is closed). The engine archive 1312 is the
  v0.64.8 cut: stdlib-v0.64.8 is live; dry-run 39283297787 is in progress on
  final pin `39f1fddff`; tag follows it. Fixed-but-acked closure
  is reporter-lane work (the compiler cannot move fixed->verified):
  reporters fill their rows in
  `release-checks/compiler-compiler-v0.64.7.md` (hash/Mutex, Float64 dlsym,
  Result-ref, platform consts, arm64, macOS memset build, cold lanes, E001
  same-call). Performance track 1: m369 landed (index
  50.6 s -> 0.7 s); the next measured split on the 6-line probe is check
  1.6 s + codegen 1.7 s, so profile the checker/codegen path next. Track 2:
  m368 landed; the remaining leak-only shapes (escaped/aliased Vecs,
  loop-redeclared Vecs, Vec struct fields) are the next runtime/codegen
  follow-up.
- CONSTRAINTS unchanged: rustc-first study protocol; max 3-4 heavy sessions;
  watch host memory (128 GB box, 32 logical CPUs); do not touch the stdlib
  repo E:\xiom-lang\stdlib except via the relay; no release re-cuts without
  owner GO.

*** SESSION UPDATE 2026-10-18T15:40Z (compiler lane; v0.64.7 SHIPPED -- CONTINUATION PROMPT) -- READ FIRST ***
- STATE: compiler **v0.64.7 released** (2026-10-17T22:59Z, 11 assets incl
  `xiom-0.64.7-linux-arm64.tar.gz`); **stdlib-v0.64.7 released**; **registry
  xiom-std 0.64.7 published** (2026-10-18T14:13Z, compiler pin v0.64.7, tag
  `f4482b1`). Compiler main `376844dd9` (post-release e2e test fixes: g15
  Win64 sret branch + m17 fixture mini_std W001 allowance). Release-notes JSON
  is live for the website. Cleanup done: 11 branches remain (main, selfhost,
  7 rustc-study, bench); E: free 158 GB; repo-local `stdlib/` at `f4482b1`.
  This session is at its context limit -- continue from the prompt below.
- **NEXT-SESSION PROMPT (copy/paste, self-contained)**:
  "You are the compiler-lane successor for XIOM (E:\\xiom-lang\\xiom). v0.64.7
  is shipped: compiler release v0.64.7 (11 assets incl. linux-arm64),
  stdlib-v0.64.7 released, registry xiom-std 0.64.7 published with compiler
  pin v0.64.7; main `376844dd9`; do NOT redo the release. Read
  docs/SESSION-HANDOFF.md top block and
  release-checks/compiler-compiler-v0.64.7.md, then:
  1) Re-verify REL-20261011-2032-engine (Vec drop not reclaimed; 15.9 GB/10 s
     at v0.64.4, not retested) on the v0.64.7 archive; if red, open a
     runtime/allocator fixer.
  2) Relaunch m360 (Int128 division AV, REL-20261016-2331-stdlib, witness
     tools/known_failures/p_i128_div_vec_loop_av.xi) -- its previous session was
     never registered.
  3) Queue/triage m365 (dep-object cache key + HIT existence race,
     1355-engine), m366 (capture-frames run-leg hang, 1128 class 2; minimized
     snippet requested), m363 (Send/Sync enforcement, THREADING_PLAN Phase B),
     m367 (catalog index excludes tests/regression fixtures).
  4) Re-triage the red-on-v0.64.7 relay items: C-ORBIT-12
     (REL-20261018-1416-orbitdb-2; Tuple__TValue__Int, 154/160), B-32/B-34/B-29
     (REL-20261017-2350-bindings; B-33 fixed), .xiomignore 1504/2104,
     alias-qualified 2100, LLVM baseline 1647, packages 2303/2309; close the
     fixed-but-acked rows.
  5) Selfhost: rebase selfhost-phase-4-codegen onto main `376844dd9`, re-run
     the RD4d/T3/T6 parity harness, adopt m37_tuple_struct (T6 13->14/64), then
     continue the remaining Phase 6 shapes.
  6) Performance: tracks 1 (compiler host) and 2 (generated code/runtime) now
     -- profile first, map hotspots to docs/rustc-study/01/02/03/07 + docs/
     RUSTC-REFERENCE.md anchors, A/B with locks; track 3 (selfhost perf) waits
     for the owner's benchmark data.
  Constraints: rustc-first study protocol; max 3-4 heavy sessions; watch host
  memory (OOM under >4 concurrent builds + e2e); do not touch the stdlib repo
  except via the relay; no release re-cuts without owner GO."
- REMAINING QUEUE (curated, highest first): (1) C-ORBIT-12 tuple unknown type
  (open HIGH, 154/160); (2) B-32/B-34/B-29 bindings cluster (B-33 fixed);
  (3) Vec-drop reclaim re-verify (HIGH, last red v0.64.4); (4) m360; (5) m365;
  (6) m366; (7) m363; (8) m367; (9) registry canary REL-20261017-2257-stdlib
  (unblocked by the v0.64.7 asset -- stdlib to run/close); (10) .xiomignore
  1504/2104; (11) alias-qualified 2100; (12) LLVM 22 baseline 1647;
  (13) packages 2303/2309 re-verify; (14) closure pass for fixed-but-acked
  rows (hash/Mutex, Float64 dlsym, Result-ref, platform consts, arm64, macOS
  memset build, cold lanes, E001 same-call).
- SELFHOST STATUS (relayed for the site as **REL-20261018-1537-compiler**):
  P5c `Vec.push` (`d82abf871`), P5d `Vec` index reads (`a0db63e1f`), P2b-iii-b
  field chains (`e3bc0261b`) landed byte-exact; RD4d parity GREEN (83-file
  check-dump + 75 manifest cases); T6 **13/64**, next `m37_tuple_struct` ->
  14/64; lane 43 ahead / 37 behind main and idle since `eb5ea46b7` -- rebase
  onto `376844dd9` first.
- PERFORMANCE (owner direction 2026-10-18): tracks **1 (compiler host)** and
  **2 (generated code/runtime)** are approved for benchmark-driven work now;
  track **3 (selfhost performance)** waits for the owner's benchmark data. The
  benchmark lane's v0.64.7 re-score confirmed the cold lanes are cold.

*** SESSION UPDATE 2026-10-18T14:22Z (compiler lane; v0.64.7 RELEASED + REGISTRY PUBLISHED) -- superseded by 15:40Z ***
- **v0.64.7 IS RELEASED.** Compiler GitHub release 2026-10-17T22:59Z (run
  39076930839 GREEN, 11 assets incl. **xiom-0.64.7-linux-arm64.tar.gz**,
  linux-x64, windows-x64, macos-x64/arm64, wasm, VSIX, SHA256SUMS). stdlib
  v0.64.7 released 22:53Z and **xiom-std v0.64.7 registered** 2026-10-18T14:13Z
  with compiler pin **v0.64.7** (OIDC trusted publish run 39135008135 SUCCESS;
  sha256 925ad7f9...). The first publish had 404'd because the stdlib tag pinned
  the candidate; under owner GO `COMPILER_VERSION` was set to v0.64.7 on
  prep+main (f4482b1/da714e4) and the tag force-moved (same name).
- FINAL BATTERY v7 (release commit 3667ee29d, stdlib-v0.64.7): e2e 2603/2605 +
  feature_regression / integration / stdlib_execution / stdlib_tests /
  api_freeze / checker ALL GREEN. The 2 e2e failures were STALE LOCKS, fixed
  post-release on main `1d79e8f9f`: `e2e_g15_sret_abi` asserted SysV semantics
  while Win64 correctly uses sret for 16B aggregate returns (now `#[cfg]`-split);
  `e2e_m17_zero_warnings` saw the m347 fixture `mini_std` duplicate `xiom.env`
  (W001) and now ignores fixture mini_std collisions. **m367** queued: exclude
  `tests/regression` fixture trees from the catalog index. Neither is a
  release regression.
- RELEASE-CHECK + bus: v0.64.7 rows closed; announcement
  REL-20261017-2259-compiler updated with the registry publish; stdlib
  coordination item REL-20261017-1444-compiler has the full chain.
- POST-RELEASE QUEUE (no release blockers): **m360** i128 division AV (its
  fixer session was never registered -- relaunch), **m365** dep-object cache
  key/race, **m366** capture-frames hang (snippet requested),
  **m363** Send/Sync enforcement, **m367** catalog-index fixture exclusion,
  **armv7** 32-bit Pi deferred (REL-20261017-1955-compiler), selfhost Phase 6
  continues in parallel.
- CLEANUP DONE (2026-10-18): removed `hold/m351-typed-fault`,
  `release/arm64-gate-check`, the m351/m353/m360/m361/m364 fixer worktrees,
  `xiom-pin-verify`, `stdlib-prep-0647` and `stdlib-verify-8d95815`, plus the
  six previously-locked orphan dirs. The repo-local `stdlib/` is checked out at
  the released `stdlib-v0.64.7` (f4482b1). 11 branches remain (main, selfhost,
  7 rustc-study, bench) + the study/selfhost/bench worktrees; E: free 158 GB.
  Candidate tags (r2) are kept for provenance.

*** SESSION UPDATE 2026-10-17T17:55Z (compiler lane; v0.64.7 CANDIDATE PUBLISHED) -- superseded by 14:22Z ***
- CANDIDATE REF: compiler main `065481eb4` + annotated tag
  **`candidate-v0.64.7-m351`** pushed; `STDLIB_VERSION` = stdlib main
  **`d6b562a`** (includes the m351 typed-fault runtime merged at `3cb1138`).
  This is the ref for stdlib's v0.64.7 release gates
  (REL-20261017-1444-compiler); the stdlib lane runs its gates, tags/releases
  stdlib v0.64.7, and replies with the tag/sha.
- WAVE LANDED + VERIFIED ON THE CANDIDATE: m351 C-ABI/fault (ffi_c_abi 2/2,
  e2e m351 2/2, m244 3/3, m308 3/3), m361 fn-value trio (6/6), m353 builtin
  shadow (7/7), m364 E001 same-call borrow (borrow_e001 4/4); `cargo check -p
  xiom --all-targets` clean; release binary builds. Earlier: m347-m350, m352,
  m359, cold-lane `--no-cache`.
- HISTORY SPLIT: the m351 commits were briefly held on local main; before the
  candidate they were re-landed on main and the docs replayed. A safety branch
  `hold/m351-typed-fault` is pushed (same content, pre-split); it can be
  deleted once the candidate is accepted.
- REMAINING TO CUT v0.64.7 (no compiler code blockers): (1) stdlib v0.64.7
  release gates on the candidate -> released tag/sha; (2) re-pin
  STDLIB_VERSION to that tag; (3) full pin battery in the isolated worktree
  (`E:\xiom-lang\xiom-pin-verify`, own stdlib, release build first, 2-3
  threads, run ALONE) -- the earlier v4 run died from host memory pressure,
  not a verdict; (4) notes/checklist verify + close the v0.64.7 check rows.
  Non-blocking in flight: m365 (dep-cache key/lock), m366 (capture-frames
  hang), m363 (Send/Sync enforcement), selfhost Phase 6 (parallel lane).
  CORRECTION: the m360 fixer session was never registered (its worktree
  exists at 007f352dd with no changes) -- relaunch m360 after the pin battery
  finishes, not in parallel with it.
- SELFHOST PROGRESS after the checkpoint prompt: P2b-iii-b field chains over
  tuple/struct values (`e3bc0261b`) + a PHASE 6 STATUS doc (`2f58b18f5`) +
  P5c Vec.push builtin for local receivers/scalar elements (`d82abf871`),
  probes byte-exact; P5d is the current WIP. T6 meter still 13/64 (next
  adoption m37_tuple_struct).
- CANDIDATE CI GREEN (`065481eb4`). v0.64.7 release notes authored and
  schema-verified: `release-notes/v0.64.7.md` + `.json` (4 highlights, 0
  breaking, user-facing known issues; `xiom-release-notes verify` green,
  commit `70a96c290`). Stdlib's prep note needs a schema fix (highlight kind
  `feature` -> `stdlib`; highlight text must be plain text, no backticks) --
  relayed on REL-20261017-1444-compiler. Pin battery v5 running at the
  candidate in `xiom-pin-verify` (own stdlib pin d6b562a, 2 threads, log
  `%TEMP%\kilo\pin-verify-battery5.log`). Remaining: stdlib gate -> released
  tag -> re-pin -> COMBINED notes verify -> final battery -> cut.
- LINUX-ARM64 RELEASE GATE IS GREEN: dry-run **39062057337** (branch
  `release/arm64-gate-check`, TEMP pin to stdlib `fix/arm64-crypto-availability`)
  -- ALL jobs green: windows-x64, linux-x64, linux-arm64, macos-x64/arm64,
  VSIX. The arm64 leg proves the native IR triple
  (`aarch64-unknown-linux-gnu`), a clang compile/link of the stdlib runtime,
  and `examples/diff_test.xi --run` exiting 42 on the arm runner, then packages
  + attests the archive. Two blockers were fixed en route: (a) compiler
  `e367ebbea` emits typed pointers in the builtin thread/channel declarations
  (opaque `ptr` is rejected by the arm runner's clang 14); (b) stdlib branch
  **`8eb22eb`** (off release/v0.64.7-prep) reports ARMv8 crypto unavailable and
  stubs the unguarded vaeseq_u8/vaesmcq_u8 block, so software AES/SHA is used
  -- THIS MUST MERGE INTO release/v0.64.7-prep + main and ride the tag
  (relayed). Also fixed for the release build: `xiom-mcp` CompileConfig
  `no_cache` (`f7575ae29`) and the public-stdlib checkout via `github.token`
  (`fd16d48f9`). Main `d6b562a` pin is unchanged; after stdlib merges/tags,
  re-pin, COMBINED notes verify, run the final locks + battery, cut.
  `release/arm64-gate-check` is a TEMP validation branch -- do not merge it;
  delete it after the final pin.
- PIN BATTERY v6 ran on main `e367ebbea` with stdlib `d6b562a` (4 threads,
  log `%TEMP%\kilo\pin-verify-battery6.log`): feature_regression PASS,
  integration PASS; e2e rc=1 (pre-hash-fix build -- superseded; the final
  battery must run on the release commit). stdlib_execution was in progress at
  the handoff.
- **STDLIB v0.64.7 GATE FAILURE ROOT-CAUSED + FIXED**: the first stdlib
  release run (39065075427) was 955/956 on BOTH OSes, failing only
  `smoke_hash_edge` (run=1, reproducible solo). Cause: m353's declared-hash
  fallthrough emitted a bare `@hash` stub for CAST/NEGATED receivers
  (`(0 as Int).hash()` -> 0) while the generic `hash[T]` called the declared
  `Int.hash` (DJB2); the smoke compares them. Fixed on compiler main
  `81f74c5a6`: `infer_expr_xiom_type_deep` carries `Expr::As` target types and
  recurses through `Unary(Neg)`, and every declared one-arg `Type.hash` scalar
  call now goes DIRECTLY through its pre-assigned symbol
  (`mono.fn_symbol_map`) instead of the ambiguous regular dispatch. Verified:
  the UNCHANGED stdlib `smoke_hash_edge` exits 0 (cast, negated, bool forms),
  e2e m353 7/7, e2e hash 4/4, stdlib_execution 85/85, feature_regression
  549/549. New candidate tag **`candidate-v0.64.7-m351-r2`** (sha
  `81f74c5a6`, includes the arm64 typed-pointer fix). stdlib asked to set
  `COMPILER_VERSION` to r2 and re-run the release gates; their `5686029`
  workaround (`0.hash()`) can stay or be reverted (both pass). Then tag ->
  re-pin -> final battery -> cut.

*** SESSION UPDATE 2026-10-17T13:20Z (compiler lane; MIDDAY+ -- pin merged, m352/m359/cold-lane landed, cleanup done) -- superseded by 17:55Z ***
- PIN MERGED: **xiom-lang/xiom#7 REBASE-MERGED** 2026-10-17T12:48Z (`8fdc3292`
  on main: STDLIB_VERSION -> `stdlib-v0.64.6`). CI fully green (ubuntu rerun
  7m19s after the pre-existing all-modules order flake; the flake reproduced at
  the OLD pin too -- filed as a residual). Pin battery v2 (clean stdlib
  worktree `E:\xiom-lang\stdlib-verify-8d95815`) was INVALID -- pointing
  XIOM_STDLIB at an external stdlib while the repo-local `stdlib/` also exists
  makes the checker index DUPLICATE stdlib roots (W001 module collisions) and
  reddens warning-sensitive e2e locks (m17 etc.). Correct setup: an isolated
  compiler worktree with its own pinned `stdlib/` and NO XIOM_STDLIB:
  `E:\xiom-lang\xiom-pin-verify` (detached at `007f352dd`, nested stdlib
  worktree at 8d95815). `cargo build --release -p xiom` FIRST -- the e2e
  harness invokes `target/release/xiom.exe`. Battery v4 running there (3
  threads): e2e / feature / integration / stdlib suites / api-freeze / checker;
  log `%TEMP%\kilo\pin-verify-battery4.log`. Delete `xiom-pin-verify` after
  recording. Record the result on REL-20261016-2330-stdlib when green.
- LANDED ON MAIN (`eff579c9`, pushed): **m252 follow-up for the benchmark cold
  lanes** (REL-20261011-2100-benchmark, owner option (a)) -- `--no-cache` now
  disables the persistent catalog index cache (`catidx.txt`) and the
  catalog-body check cache (`checkbodies.txt`) READ+WRITE in the compile, AOT
  run and in-process JIT paths (`CompileConfig::no_cache`,
  `jit_execute_with`). Lock `bench_no_cache_catalog_caches` wired into ci.yml;
  m252 parity 1/1, run_script_cli 8/8, JIT lib 4/4. Benchmark notified
  (REL-20261017-1304-compiler) to re-score xiom-run-jit/aot.
- M351 HARVESTED LOCALLY, HELD FOR THE STDLIB RUNTIME: 3 commits cherry-picked
  on local main (`fcb38d7ef` fix: new c_abi.rs + call/expr/decl/emitter/lib,
  Win64 C-ABI aggregates + typed confined-fault exit-70; `a279d8679` tests:
  in-repo ros2-abi ladder/direct-extern/IR twin/no-silent-exit + e2e locks;
  `d745fab38` docs). Locks GREEN with the m351 binary: ffi_c_abi_aggregate 2/2,
  e2e m351 2/2, m244 3/3, m308_guard 3/3 (the earlier isolated-target run
  failed only because the e2e harness resolves `target/debug/xiom.exe`; do not
  use CARGO_TARGET_DIR for e2e). **NOT PUSHED**: the runtime half lives in the
  stdlib repo -- branch `fix/m351-typed-fault` (pushed; HEAD `37603a2`, rebased
  onto stdlib main `8ce84cc`: runtime TLS fault record + always-on diagnostic + exit 70 + typed
  `xiom.error.take_fault` + retry only for guard-page transients). The released
  pin 8d95815 lacks `xiom_fault_*`, so a compiler pinned there would fail to
  LINK confined-block programs. Coordination item
  **REL-20261017-1444-compiler** (to stdlib, HIGH): rebase onto stdlib main
  (waves 172-177), include in the stdlib v0.64.7 release, reply with tag/sha;
  then pin STDLIB_VERSION, re-run the pin battery, push m351.
- RELAY INTAKE TRIAGED (all acked + pushed): 1223-orbitdb (transient exit -1
  = host memory pressure evidence: rustc E0786 paging-file + rc -1 spawn
  failures on this box; capture-a-dump ask), **1223-orbitdb-2 RECON COMPLETE
  -> m364 LAUNCHED** (exact repro `--emit-ir` on
  `xiom-orbitdb/tests/test_conformance.xi:4518`; one borrow site
  double-reported; trigger = by-value arg then borrow of the same local in a
  later arg of the SAME call; check_call marks bare idents moved eagerly,
  codegen evaluates args before the call and Vecs pass as copies -> false
  positive, m344-era; minimal 2-line shape; fix = defer the move until all
  args are checked with a pending-move set; `--check` skips the borrow pass =
  parity residual; no orbitdb workaround needed), 1227-packages (HIGH
  fn-value trio -> **m361 LAUNCHED**, branch fix/m361-fn-value-trio).
  **m353 LAUNCHED** (builtin shadow), **m360 LAUNCHED** (i128 div AV), m364
  launched. Heavy slots: selfhost Phase 6 + m361 + m353 + m360 + m364.
  MEMORY NOTE: five concurrent cargo builds + the e2e battery exhausted the
  system commit (PowerShell 800705af, rustc E0786); the battery was stopped
  and its orphan tree killed. Run the pin battery v4 alone (or with <=2
  fixers) to get a clean signal; the e2e failures seen under exhaustion
  (m20/m33 etc.) are not verdicts.
- SELFHOST PHASE 6 (parallel lane; 38 commits ahead of origin/main, 0 behind):
  RD4d parity is GREEN -- 83-file check-dump corpus + 75 manifest cases, 11
  non-vacuous diagnostic lines (`tmp/sprintc/phase6/diff_check_rd4d.log`, 32
  min run). P2b-i tuple literals and P2b-iii-a tuple field reads (.0) +
  tuple-typed locals landed byte-exact (`7992b124b`, rebased; originally
  `f7e2585d4`). T6 meter 13/64; next adoption target `m37_tuple_struct` ->
  14/64. The lane went idle with an UNCOMMITTED P2b-iii-b WIP in
  `selfhost/src/codegen.xi` (general base-expression field chains like
  `dm.0.d` + NkExprStruct tuple element names), untested since 15:14Z; it has
  been prompted to probe/commit-or-revert, append a PHASE 6 STATUS block to
  docs/SELFHOST_PROGRESS.md, and continue.
- WORKTREE/BRANCH CLEANUP: 65 -> 14 git worktrees; 54 completed fixer
  branches deleted; stale pin branches (v0.64.5/v0.61.3) deleted; selfhost
  phases 1-3 removed (0 unique commits); disk free on E: 190 GB. Kept: main,
  m351 (live), selfhost-phase-4 (live), 7 rustc-study worktrees, xiom-bench.
  Six deregistered dirs (m347-m350, m352, m359) are still file-locked by the
  VS Code extension; delete after an extension reload (their branches/registry
  are already gone).
- STUDY PROTOCOL (owner directive 2026-10-17, now enforced): `E:\repos\rust`
  (READ-ONLY sparse clone @ 7211ea594) is the battle-tested reference. Every
  design/perf/scaling/selfhost-gap/bug fix must (1) read the matching
  `docs/rustc-study/0N-*.md` area doc, (2) crawl the rustc source for the
  mechanism that already solves it, (3) record the rustc anchor in the
  COMPILER_BUGS/decision entry, (4) use benchmark relays as study questions.
  Written into `docs/RUSTC-STUDY.md` ("Protocol -- rustc-first"); both live
  fixer sessions (m361/m353) received the directive and must cite anchors.
- THREAD POOL OWNERSHIP (owner question): 1834-engine-6 is stdlib's (pool flip
  rides 1550-stdlib); the compiler-owned spawn gateway 2053 is FIXED (m282 +
  runtime wave 146 -- engine to re-verify); the remaining compiler-owned piece
  is Send/Sync ENFORCEMENT (THREADING_PLAN Phase B) -> queued **m363**.
  Recorded on REL-20261017-1438-compiler.
- ENGINE RELAY DELTA (2026-10-17 evening): REL-20261017-0359-engine (cross-
  module Mesh types) -> FIXED, engine verified v0.64.6 (probe_obj 12/12 rc 0;
  rename workaround retired). REL-20261016-1128-engine class 4 rc-101 is
  EXPECTED on the v0.64.6 archive (pre-m359) -- m359 is on main; re-test next
  candidate. NEW class 2: probe_capture_frames RUN-LEG HANG at v0.64.6 (no
  stdout, 6-11 MB WS, killed 60/92/180 s; 0.64.4 PASSes 10 checks) -- minimized
  capture-only snippet requested -> queued **m366**. REL-20261017-1355-engine
  addendum: m340 dep-object cache HIT does not verify object existence (race ->
  hard clang link failure) and the key is consumer-specific (139-TU assimp
  rebuild per target) -> queued **m365** (shared dep key + HIT existence check +
  per-key build/evict lock; Stage 6 cache). Both queued behind the live wave.
- RELAY ACKS RECEIVED: benchmark acked REL-20261017-1304 -- re-scores
  xiom-run-jit/aot on the FIRST release archive carrying eff579c9 (v0.64.7),
  checks no catidx/checkbodies under --no-cache, one lane at a time. Bindings
  acked REL-20261017-1245 -- B-33 verdict awaits m351; B-29/W1.4/W2.3/B-34
  remain queued on their re-verify list. STDLIB has NOT acked
  REL-20261017-1444-compiler yet (runtime c21ad9f still must ride v0.64.7);
  the m351 push stays held.
- The 11:10Z block below retains the morning context; superseded where it
  disagrees.

*** SESSION UPDATE 2026-10-17T12:45Z (compiler lane; MIDDAY -- pin verify + m352/m359 harvested) -- superseded by 13:20Z ***
- PIN (REL-20261016-2330-stdlib): stdlib-v0.64.6 moved to 8d95815 and PUBLISHED
  (xiom-std 0.64.6 live; publish 38979638142 SUCCESS 23:57Z). Pin PR
  **xiom-lang/xiom#7** open; the auto-bot commit lacked DCO sign-off (amended
  `d65654bd` with the bot sign-off; DCO green). First PR CI run fully GREEN; the
  re-run hit a **pre-existing all-modules-compile ORDER flake** (the freeze
  test builds `use xiom.X;` from a HashSet -> random import order; the same
  test also flaked at the OLD pin b81430ba; reproduced 1/3 old vs 3/3 pass new
  on clean checkouts). ubuntu job re-running. Pin battery at 8d95815 running on
  a CLEAN stdlib worktree (E:\xiom-lang\stdlib-verify-8d95815; the shared
  repo-local `stdlib/` is being edited by the m351 session and must not be
  used for verification): e2e / feature / integration / stdlib suites /
  checker, log %TEMP%\kilo\pin-verify-battery.log. Merge PR #7 per the 1559
  protocol when the rerun is green; record the battery result on 2330.
- HARVESTED ON MAIN (pushed `883f69bf`): m352 `001570e9` + `7d1a9cd2`
  (host-arch Target::Native table + toolchain_cmd linux-arm64 arms; unit lock
  green) and m359 `736f876f` + `883f69bf` (duplicate value flags after the
  positional -> rc 2, tolerant clap accessors; 2-test lock wired into ci.yml;
  bin unit 8/8, lock 2/2). m351 (B-33) still in flight (3 commits + uncommitted
  docs/tests in its worktree): monitor, do not harvest yet.
- SELFHOST PHASE 6 (parallel, ses_ebeb8f547ffeSvMCBgei6xvota): T6 meter 13/64;
  P2a/P2b-i tuple-name + tuple-literal parity landed (`aa62a2d1`); lane rebased
  onto v0.64.6 release main 19eb7218 (36 ahead); continuing the struct/tuple/
  generic corpus. Status relay: REL-20261017-1245-compiler.
- RELAY INTAKE (new, triage next): REL-20261017-1223-orbitdb (transient
  compiler exit -1, no diagnostics, under host load; rerun green -- exit-path/
  OOM audit + dump recipe), REL-20261017-1223-orbitdb-2 (E001 false positive on
  borrowed &Vec[Int] reads; pairs the m344 E001 coverage), REL-20261017-1227-
  packages (HIGH: module-global fn var codegen, adjacent fn struct fields
  layout, fn-alias callable param AV). Queued slots m353-m360 remain; m352/
  m359 landing frees two heavy slots.
- STAGE 6 (perf/lint): benchmark t3 row (~230 ms vs C 25 ms) still awaits a
  profile; slotted behind the v0.64.7 wave. Worktree cleanup of the integrated
  m29x-m350 fixer worktrees/branches is in progress (live m351/m352/m359 +
  selfhost/rustc-study worktrees preserved).
- Relay status item: REL-20261017-1245-compiler (to: all) -- the website lane
  reads the bus.

*** SESSION UPDATE 2026-10-17T11:10Z (compiler lane; MORNING RESUME -- context handover) -- READ FIRST ***
- CONTEXT WINDOW FULL in the previous session (this block supersedes the
  2026-10-16T23:22Z PAUSED block; keep it for reference).
- MORNING STATE: v0.64.6 released 2026-10-16T20:33Z (run 38962236697) and is
  VERIFIED by upstream: orbitdb 2022 CLOSED (verified on v0.64.6); xvector
  Windows legs green (C-17 7/7, C-18 green; C-16 WSL deferred by env + owner
  WSL pause); stdlib 2100 W2-W4 fixed (W1 residual -> m357); benchmark absorbed
  (green); benchmark + packages closures recorded.
- WAVES INTEGRATED ON MAIN (compiler repo, all pushed): m334-m346 (the v0.64.6
  wave) + m347 (platform consts target-value + seed poison) + m348 (fn-ptr
  return ABI flow) + m349 (Array injection + reserved-name diagnostic) +
  **m350 (Result-ref field reads; just harvested: fa475ddd/1a84767e picked as
  e4557ce0/9fb10c40; wave locks 8/8 on the integrated tree; full locks 39/39
  before m350)**. Relay records: f15c01c + a7d349d.
- LIVE SESSIONS (Agent Manager, may finish while this session ends -- HARVEST
  THEM FIRST in the next session):
  * m359 CLI duplicate-flag clap panic -- session ses_eb92e43cbffeiQbgJ9ZYhbCp4e,
    worktree fix-m359-cli-clap-duplicate-flags-*. Was EMPTY at resume; nudged to
    begin (11:12Z). Closes REL-20261016-1128-engine class 4. Definition in
    %TEMP%\kilo\engine-round2-recon-report.md finding 3.
  * m352 arm64 native triple -- THIRD spawn requested (am-1792235123971-5w1aom)
    after two silent failures. If no worktree appears: IMPLEMENT DIRECTLY in a
    manual worktree (git worktree add .kilo/worktrees/m352-manual -b
    fix/m352-arch-native-triple): arch-aware Target::Native (crates/xiom/src/
    lib.rs:1400-1411 + ixom-codegen/src/context.rs:186-194; aarch64 linux ->
    aarch64-unknown-linux-gnu, aarch64 macos -> aarch64-apple-darwin) with a
    pure (os,arch) table test + toolchain_cmd.rs linux-arm64 asset/sums arms
    (+unit tests ~286-298) + optional nasm x86_64 gating. Then add the
    functional on-runner native-compile gate to the release.yml linux-arm64 leg
    and dispatch a workflow_dispatch dry-run.
  * B-33 recon COMPLETE (report C:\Users\lefte\AppData\Local\Temp\kilo\recon-b33\REPORT.md,
    543 lines; artifacts + register proofs + naked-vs-clang IR/asm under
    %TEMP%\kilo\recon-b33\). Verdict: TWO composed defects -- (1) Win64 C-ABI
    aggregate lowering is wrong at extern + raw-Int-as-fn boundaries (16B
    by-value rcx:rdx vs C pointer-in-rcx; 40B 5 words vs pointer; 16B return
    rax:rdx vs sret), defect matrix + file:line map in the report; (2) the
    confined fault path turns any AV into a SILENT exit-0 (SEH catches; the
    language fault is just not represented; P1 CI masking; secondary:
    uninitialized alloca on struct-return fault paths, attempt trace always 0,
    deterministic faults retried -> side effects twice). The item's in-repo
    bundle claim was WRONG -- the bundle must be landed by the fixer.
    **m351 LAUNCHED** (fix/m351-ffi-aggregate-abi; brief = the REPORT.md
    sections 5-7; 4 commits: codegen CAbiPlan + runtime typed fault +
    docs + tests; locks: in-repo probe ladder e2e, direct-extern e2e, IR twin
    no-naked-%struct/sret, no-silent-exit exit-70 + take_fault exit-0; update
    the m244/m308 exit-0 assertions; target-dispatch must not regress
    SysV/AArch64).
  * selfhost Phase 6 (ses_ebeb8f547ffeSvMCBgei6xvota): nudged 11:12Z to rebase
    onto current main (was 8 behind) and advance; verify parity against m347-m350.
- STDLIB SYNC (owner rule, HARD): NO v0.64.7 candidate until the stdlib v0.64.7
  release is prepared with markers bumped AND approved/published, and
  STDLIB_VERSION references that released rev. Current status: stdlib v0.64.6
  tag moved to 8d95815 (waves 155-171); stdlib block 224 = PAUSED by the owner;
  registry publish 38979638142 STILL AWAITING OWNER APPROVAL; the release.yml
  auto pin PR (STDLIB_VERSION -> 8d95815) has NOT appeared yet (gh pr list
  empty at 11:05Z). When it appears: merge per the 1559 protocol + run the
  archive-level verify on the resulting revision; record on REL-20261016-2330.
- QUEUED FIXER SLOTS (v0.64.7, launch as machine slots free; max 3-4 heavy):
  m351 (B-33), m352 (arm64 triple), m353 (builtin-shadow sweep: stdlib 1053
  .hash + 1456 Mutex barrier), m354 (1626 trailing-match T001), m355 (2343 bare
  inline i32-extern sign), m356 (2328 catalog re-add recon), m357 (2100 W1
  non-bare field spellings/method dispatch), m358 (B-32 in-situ sqlite; m346
  extension), m359 (CLI, in flight), m360 (i128 division AV recon), plus the
  older acked backlog wave (packages 1504 re-queue, playground rows, engine
  classes 2/4, XVC-C-08/09/14 family).
- OWNER-DECISION QUEUE: engine-2134 finding 5 (consume-vs-copy rule after the
  m318 pin), m252 cold-lane, the 5 compiler->owner items (Oct 10), B-33
  severity flow, WSL pause (C-16/C-17-WSL close-gates).
- PROCESS CHEATSHEET (hard-won this session):
  * Cherry-pick conflict pattern: ci.yml -> `git checkout --ours` + append the
    fixer's e2e tokens at the line end; COMPILER_BUGS.md -> union (remove the 3
    markers; keep both sides) and insert `    );\n}` + a blank line at joins
    where a side ends mid-function; e2e_tests.rs -> union with the SAME brace
    repair (a side ending at an assert message line needs `    );\n}` inserted
    before the next block). Always run the marker grep + `git diff --cached
    --check` before `--continue`.
  * Relay commits: a concurrent lane push can reject ours -> manual `git fetch
    origin` + `git push origin main` (never stage other lanes' untracked items
    in the shared clone; never `git add .`).
  * WSL: default distro lacks bash -- use `wsl -d Ubuntu -e bash -c "..."`; a
    raw aarch64 binary cannot run on this x64 box (qemu absent) -- `file` is
    the check.
  * release.yml: tag-guarded (tag == workspace version, ancestor of main);
    workflow_dispatch = dry-run artifacts; notes verify limits: 6 highlights,
    summary 240, title 60, text 320 chars, no internal task ids (m###) in text;
    required MD sections Summary/Highlights/Breaking changes/Known issues/Docs.
  * arm64 leg: matrix on ubuntu-22.04-arm + z3 linux-arm64 pin
    (4.13.4-arm64-glibc-2.34, sha 37533e7a...); wasm gated to linux-x64;
    per-platform archive naming + SHA256SUMS (commit 19eb7218).
- WATCH: the */15 cron is DELETED (pause + context handover); re-arm in the
  next session by re-scheduling the "Post-cut wave + relay watch v2" prompt
  (in session history) or sweep manually. Zero open compiler items at 11:05Z
  besides the acked queue; relay head a7d349d.

*** SESSION PAUSED 2026-10-16T23:22Z (owner away; resumes 2026-10-17) -- historical, superseded by the morning block above ***
- Watch DISARMED (the */15 cron deleted). In-flight Agent Manager fixer
  sessions continue on their own in their worktrees: m350
  (fix/m350-result-ref-field-read; recon %TEMP%\kilo\m350-recon-report.md) and
  m359 (fix/m359-cli-clap-duplicate-flags; closes 1128 class 4) were mid-battery
  at pause -- their commits wait on their branches for harvest. m352
  (fix/m352-arch-native-triple) had NO worktree at pause -- RE-REQUEST on
  resume (its first two spawn requests silently failed; check the AM list).
  m351 recon (B-33) was running in-session; its report lands under
  %TEMP%\kilo\recon-b33 when done.
- RESUME CHECKLIST (next session):
  1) Harvest m350/m359/m352 (+ m351 recon verdict) with the standard pattern:
     verify, cherry-pick (ci.yml / COMPILER_BUGS / e2e_tests union + brace care
     at joins), lock suite, relay notes + plan rows, commit.
  2) Stdlib pin PR watch: release.yml auto-files STDLIB_VERSION -> stdlib-v0.64.6
     (8d95815) -- merge per the 1559 protocol + run the archive-level verify
     (item REL-20261016-2330-stdlib).
  3) arm64: after m352 lands, add the on-runner native-compile gate to the
     release.yml linux-arm64 leg + dispatch a dry-run; record on
     REL-20261016-1917.
  4) Queued fixer slots: m358 (B-32 in-situ residual), m360 (i128 division AV),
     m353 (builtin-shadow sweep 1053+1456), m354 (1626), m355 (2343),
     m356 (2328 recon), m357 (2100 W1).
  5) RELEASE RULE (owner): NO v0.64.7 candidate until the stdlib v0.64.7
     release is prepared with markers bumped AND approved/published, with
     STDLIB_VERSION referencing that released rev.
  6) Relay watch: re-arm a cron (the v2 prompt is in the session history) or
     sweep manually; zero open compiler items at pause.
  7) Owner-decision queue: engine-2134 finding 5 (consume-vs-copy), m252
     cold-lane, the 5 compiler->owner items (Oct 10), B-33 severity flow,
     1504 re-queue, playground rows, WSL pause (C-16/WSL close-gates).
- State at pause: v0.64.6 released + verified; wave m347/m348/m349 integrated
  (locks 39/39 + checker 1/1 + ffi 2/2); compiler main pushed (ee62e817-era);
  relay pushed (9ebc3a5-era); stdlib-sync gate recorded in three places.

*** SESSION UPDATE 2026-10-16T23:25Z (compiler lane; POST-v0.64.6, release-protocol correction) -- READ FIRST ***
- OWNER PROTOCOL CORRECTION (23:12Z): do NOT cut/release a compiler candidate
  unless the stdlib is on the SAME TRACK -- the stdlib release for the same
  version must be prepared AND approved/published first (version markers bumped,
  tag on the released rev). v0.64.6's archive pinned stdlib b81430ba whose
  package.xi still said version "0.64.5", so installs packed a 0.64.5-labeled
  stdlib; the stdlib lane is now releasing 0.64.6 to match (block 223: prep
  8d95815, tag MOVED to 8d95815, release run 38979638226, publish 38979638142
  awaiting owner approval). HARD RULE for v0.64.7: cut only after (a) the
  stdlib v0.64.7 release is prepared to the same version with markers bumped,
  (b) its publish is approved, (c) STDLIB_VERSION references that released rev.
- STATE: v0.64.6 RELEASED 20:33Z (release run 38962236697 green; assets
  verified). Upstream closures: orbitdb 2022 CLOSED (verified on v0.64.6);
  xvector Windows legs green (C-17 7/7, C-18 green; C-16 WSL leg deferred by
  environment + owner WSL pause); stdlib 2100 W2-W4 fixed on v0.64.6 (W1
  residual -> m357, medium); benchmark absorbed (green, pin 6d2c142);
  packages 1941/1304 closed. Wave 2 integrated: m347 (platform consts
  register the TARGET value + seed poison removed), m348 (fn-ptr return ABI
  flow for call-valued/Ident-copy bindings), m349 (Array catalog injection +
  reserved-name diagnostic) -- commits d266a080..5cb7515c; wave e2e locks
  39/39 + checker 1/1 + ffi 2/2. Relay record f15c01c.
- IN FLIGHT: m350 (Result-ref field read crash; recon %TEMP%\kilo\m350-recon-report.md),
  m359 (CLI duplicate --timeout/--jobs panic -> closes 1128 class 4),
  m352 (arch-aware native triple + toolchain_cmd linux-arm64; RETRY spawn --
  first request silently failed), m351 recon (B-33 confined-call silent
  exit-0; report %TEMP%\kilo\recon-b33), selfhost Phase 6 (rebased onto
  release main; advancing; T6 meter report pending).
- ARM64 (REL-20261016-1917): release leg MERGED (19eb7218) + arm64 z3 pin
  (4.13.4 glibc-2.34, sha verified); dry-run 38967096669 GREEN incl.
  package linux-arm64; artifact aarch64-verified; functional on-runner
  native-compile gate + m352 remain; ships in v0.64.7 (owner call);
  packages Pi smoke x2 = acceptance.
- OWNER-DECISION QUEUE: engine-2134 finding 5 (consume-vs-copy rule; runtime
  ABI copies -- only the diagnostic is live), m252 cold-lane (2100-benchmark),
  the 5 compiler->owner items from Oct 10, B-33 severity flow (m351),
  registry 1504 re-queue (slotted for v0.64.7), playground rows still red
  (backlog wave), WSL pause (C-16/C-17-WSL close-gates).
- WATCH: a */15 cron "Post-cut wave + relay watch" is ACTIVE
  (wku_1467905dc001BkHXug14AsOM2W) -- relay sweeps + fixer harvest + arm64
  follow-ups; re-arm in a fresh session if this one ends.
- PROCESS unchanged otherwise: fixers commit on their branches, the main lane
  cherry-picks + owns relay rows; main pushes are normal now (release path).

*** SESSION UPDATE 2026-10-16T11:55Z (same compiler session as the wrap below; owner directive + wave A started) ***
- OWNER DIRECTIVE (11:44Z): ALL incoming relay intake from pulse, engine,
  orbitdb and xvector is to be addressed to 100% coverage in the next waves;
  agents allowed; items added to the next candidate list.
- DONE THIS SESSION:
  * Engine intake REL-20261016-1128-engine acked with a four-class triage
    (relay commit e80e382, pushed).
  * Candidate list created: relay release-checks/compiler-compiler-v0.64.6.md
    (check-new compiler-v0.64.6) with a Coverage-plan section assigning every
    four-lane item to wave A/B (relay commit d4da0ea, pushed). 118 rows total.
  * m334 re-test round COMPLETE (verdicts committed on the items):
    1941 GREEN on the rebuilt m334 exe (5784c7c3; probe_tuple + control +
    module-boundary all green); C-ORBIT-12 (2022) STILL RED (same clang
    two-name split); 2303 all 16 probe shapes unchanged vs v0.64.5; 2309
    STILL RED with three clean repros on record: probe_c.xi (socket path),
    NEW user-level probe_d3_result_value.xi (single-file Result.value
    destructure) and probe_d4_main.xi (module-boundary) all warn
    unknown type Tuple__Vec[UInt8]__Str__Int + misdecode (len=6, pointer-like
    peer/port); the match-arm and plain field-access variants are green, so
    the trigger is Result.value payload access for tuples with Vec elements.
    Repro scratch: %TEMP%\kilo\m334-retest\transport\.
  * Wave A recon IN FLIGHT (4 read-only Task agents): orbitdb-2022 namer root
    cause; xvector-2216 imported-type ptr-field stores; xvector-2142 linux
    /search localization; engine-1128 classes 1-4. Round 2 queued: engine
    1358/1538/1046/1411, pulse 1549-pulse-2/3/4.
- NEXT: AM fixer sessions per triaged item (m336+ numbering; wave process owns
  merges); m334/m335 integration into local main still pending -- do NOT
  integrate while recon builds are running.

*** SESSION WRAP 2026-10-16T11:30Z (compiler lane; context moving to the next session) -- READ THIS FIRST ***
- Last session's scope: the v0.64.6 general-Darwin stream (COMPLETE, CI-green; the
  23:40Z block below carries the fix/evidence detail) + ad-hoc compiler-lane duty:
  relay intakes, the m334/m335 fixer wave, and the selfhost Phase 6 kick-off.
- STATE AT WRAP (all verified):
  * stdlib: 2bc064e + 9d6a4d5 on main; e72288f (win64 fp128 convention split:
    clang>=21 pointer shims / clang<=20 XMM shims) is on the witness branch
    witness/darwin-runtime-v0.64.6 and NOT merged -- TOP OPEN LOOP: the stdlib
    lane should merge e72288f at their next wave (relay 2011 says so); the
    branch is disposable afterwards. Waves 145/146 (real OS threads) landed on
    stdlib main overnight.
  * Compiler repo: local main = 9096c711 + this wrap commit (docs only,
    UNPUSHED per constraints). Fixer branches done, not merged (the wave
    process merges): m334 fix/m334-tuple-name-collision (b5b97427 + 5784c7c3;
    tuple-name collision via the from_utf8 Call-arm type inference), m335
    m335-bigint-bitops (d234d792 + e50d79dd; B-29 triaged to the bindings
    probe's own literal mis-conversion -- big-int bitops are EXACT; correction
    filed on REL-20261014-1208-bindings-2 with a reporter re-run requested).
  * Relay: zero open+unacked compiler items. Fixer queue (all acked, to
    schedule): C-ORBIT-12 (REL-20261015-2022; still red on the official
    artifacts per the orbitdb check), XVC-C-16 (2142; linux /search
    regression), XVC-C-17 (2216), 1941 (fixed by m334 -- re-test C-ORBIT-12 +
    2303/2309 against the m334 branch FIRST), 2303 enum-variant defects, 2309
    mixed-element tuples, 2309-2 bare-assert mis-bind, 2328 stdlib-6 residuals,
    2343 bare-inline i32 misread, B-32 (2130), plus the older acked set
    (1504/1509/1358/1538/1109/2032...). Open finding: __fixdfti (Float128 as
    Int128 links the missing double->i128 helper on win64).
  * Selfhost: Phase 6 advanced overnight (rebase-drift port, P5b, tuple-name
    P2a byte-exact; REL-20261013-1732 FIXED + standalone repro filed). Lane
    home: AM worktree selfhost-phase-4-codegen; check the AM list for its
    current state.
  * Watch: the hourly relay-watch cron is session-scoped and dies with this
    session's goal -- a next session should re-arm a watch if desired.
- NEXT SESSION: start from this block + the 23:40Z block below; the stdlib lane
  merge of e72288f and the fixer queue are the two live threads; everything
  else is recorded on the relay (2011/2013 and the intake items).

*** v0.64.6 DARWIN STREAM COMPLETE 2026-10-15T23:40Z -- READ THIS FIRST ***
- Scope (owner decision per relay 1549/1609-pulse; this session's goal): the
  general-Darwin workstream. ALL THREE NAMED RUNTIME GAPS FIXED; the engine
  ask is relayed and acked. Nothing else in flight from this stream.
- stdlib fixes (on stdlib main unless noted):
  * 2bc064e -- fp128 darwin ABI: the whole f128 libcall family defined for
    __APPLE__ x86_64+arm64 with 16-byte-vector C signatures (Apple clang has
    no f128 C type; the asm shims never materialized -> macos-x64 linked the
    tf3 family UNDEFINED; REL-20261010-1609-pulse-2). mach host_statistics64
    free-memory fallback in xiom_runtime.c (the 1549-pulse-7 follow-up; the
    -1 sentinel is gone). PRE-EXISTING fp128 opposite-sign addsub + zero
    fast-path corrections (all platforms; shift-UB + exponent-term sign;
    0.0-a returned +a).
  * 9d6a4d5 -- ELF x64 takes the same vector wrappers (the old Win64 shims
    were being selected on linux and were mis-ABI: first runtime-valued
    exercise SIGSEGV'd).
  * e72288f (WITNESS BRANCH ONLY -- stdlib lane to merge at the next wave) --
    win64 fp128 convention split: clang >= 21 pointer shims; clang <= 20
    (GH runner images ship LLVM 20.1.8) XMM-convention shims. Root cause of
    the windows AV: the win64 f128 libcall lowering changed at clang 21
    (probed with real 20.1.8/21.1.8/22.1.8 toolchains; the dev box
    reproduced the runner AV exactly under 20.1.8).
- Locks: tests/smoke/smoke_darwin_runtime.xi + smoke_diag_f128_l0..l5
  (corpus 955 -> 962; runtime-valued fp128 libcalls; macOS free-memory
  contract). All green on every leg.
- CI EVIDENCE (heavy run 38819521229 on witness/darwin-runtime-v0.64.6,
  compiler_ref=v0.64.5): windows-latest 962/962 GREEN; ubuntu-latest
  962/962 GREEN; macos-14 957/962 -- the remaining five are OUT of this
  stream (guard_fault = the POSIX fault-trace item, section 2 below;
  platform / ffi2 / simd compilefail / stress_crypto_aes_roundtrip =
  stdlib-lane items). The darwin lock passes on macos arm64 end-to-end.
- Records: relay REL-20261015-2011-compiler (fixes + verdicts; includes the
  lane action to merge e72288f); engine ask REL-20261015-2013-compiler
  (engine lane ACKED -- upstream gate cleared + macOS assessment recipe;
  their macOS run is the engine-side step, not ours).
- OPEN FINDING (recorded, not fixed): runtime `Float128 as Int128` links
  __fixdfti (double -> i128) on win64 -- the runtime does not ship it.
- Local fp128 toolchains: LLVM 20.1.8 + 21.1.8 extracted under
  %TEMP%\kilo\llvm20\ / llvm21\ (PATH-prepend + XIOM_RUNTIME_DIR overlay
  reproduces the full runner clang matrix locally).
- Companion lanes: selfhost Phase 6 session running (AM worktree
  selfhost-phase-4-codegen); fixers m334 (DONE: b5b97427 + 5784c7c3) and
  m335 (B-29 big-int bitops, running). The stdlib lane owns stdlib main.

CURRENT STATE (2026-10-13T20:30Z; read this before anything below):

*** v0.64.5 RELEASED 2026-10-15T19:20Z -- READ THIS FIRST ***
- RELEASE: tag `v0.64.5` (annotated) on merged main `84f761c5` (pin
  commit: STDLIB_VERSION = stdlib-v0.64.5 sha `092dfa0` (tag obj
  1667b3a8) + release-notes/v0.64.5.json (6 highlights) + gate record).
  Pin PR xiom#6 rebase-merged after PR CI round 4 all green (ubuntu +
  windows + workspace hygiene); tag release run `38792051847` FULLY
  GREEN incl. the publish job; GitHub Release v0.64.5 LIVE + non-draft
  (SHA256SUMS + xiom-0.64.5-{linux-x64,windows-x64,macos-x64,macos-arm64}
  + VSIX 0.12.2 + wasm bundle). stdlib side: stdlib-v0.64.5 (092dfa0) +
  xiom-std 0.64.5 published. Pin-gate fix round m328-m333 (perf budget,
  api-freeze snapshot, rustls/vet, C++ runtime link, trap-lock
  platform-awareness) shipped in the tag. Records: candidate record
  (bus `4b73d3f` + `fdaee6c`) + this file.
- LOCAL STATE: main fully pushed (local == origin; only untracked
  `m211_listdir_probe/` scratch). All Agent Manager fixer sessions idle
  (m311-m333 worktrees kept for reference). No pending wakeups needed;
  the goal completed with the release.
- NEXT SESSION WORK (v0.64.6):
  1. HEADLINE: general Darwin/macOS -- stdlib runtime items (tf x64
     symbols, `_SC_AVPHYS_PAGES`, arm64 fp128 asm) + ENGINE macOS
     build/run evidence (games); the `@llvm.memset` declare already
     shipped (m331). PULSE 1549/1609 items acked with the archive-verify
     pending on v0.64.5.
  2. POSIX fault-trace parity (m244/m308 markers are Windows-VEH-only;
     stdlib runtime) + any Linux semantics gaps the platform-aware m333
     locks now separate out.
  3. Queued acked items for triage: `REL-20261014-1208-bindings-2`
     (wide-int/confine witnesses re-run on the v0.64.5 archive),
     `1504-registry` (pkg guard / .xiomignore), `1509-engine` (repeated
     scalar flags in port.args), `1510-engine-2` (ordering adopted),
     `1109-bindings` (android cross-target scoping; owner priority),
     `1358/1538-engine` (editor W1/W2 toolchain).
  4. POST-RELEASE WAVE: archive reporter re-verification of the v0.64.5
     artifacts across lanes (bindings/orbitdb/playground/engine/stdlib
     rows per the candidate record; the 27-repro sweep expectations).
  5. Older queue: 2032 owner decision, macOS pair archive re-verify,
     1549-packages-2 faithful repro, release-gate record wrap.
- OPS: battery recipe = section 5; ops lessons = section 6; the whole
  m311-m333 fight log is in the RELEASE LEG STATUS block + section 8.
  ALWAYS: fetch `origin` AND the relays before trusting anything.
- 2026-10-16 m335 (B-29 leg of `REL-20261014-1208-bindings-2`) DONE --
  TRIAGED, NO compiler defect. The bindings bundle's bigint-bitops probe
  mis-converts its own literal: 3132620800 = 0xBAB80000, NOT 0xBAD00000
  (the real NVSDK_NGX_Result base 0xBAD00000 = 3134193664), so the values
  it reports as wrong (2987 / 3132096512 / 2987) are the arithmetically
  correct ones for the literal as written and its "want" values are
  impossible. Re-verified on the v0.64.5 archive AND on main across
  const-folded/hex/runtime-global/fn-param shapes, UInt32->Int zext +
  Int32->Int sext provenance, a runtime shift sweep, and the NGX
  `(rc & 0xFFF00000) == 0xBAD00000` loop over `0xBAD00000|N` (N 0..18);
  `--emit-ir` shows `ashr/and/sdiv/or i64` only -- no 32-bit path. Branch
  `m335-bigint-bitops` (UNPUSHED; stacked on the m334 tip 5784c7c3):
  d234d792 (green-first triage lock tests/regression/m335_bigint_bitops +
  `e2e_m335_bigint_bitops` + `_ir` twin + ci.yml g2 token + COMPILER_BUGS
  entry) and 63d3dd48 (doc: lock sha). Battery all green: locks 2/2, e2e
  g1 16 + g2 245 (x2), codegen lib 12, feature_regression 549, integration
  130; the lock reports 0 failing checks on the archive too (there is no
  red arm -- the finding was never a defect). REAL defect is package-side:
  `xiom.dlss` `NVSDK_NGX_RESULT_FAIL = 3132620800` (comment "0xBAD00000"),
  so `dlss_failed(rc) = (rc - 3132620800) in [0, 18]` reads every real
  NGX failure as success (its suite's equivalence pin shares the wrong
  constant and cannot catch it); the bundle probe.xi + the B-29 ledger row
  need the 3134193664 correction before the workaround can be retired.
  The requester's Agent Manager session closed before the reply landed --
  this block + the COMPILER_BUGS entry + the commit messages carry the
  full report.


- origin/main = `91860f11` (fetch before trusting -- the doorbell may push
  staged work). Local main = the latest commit (see git log), AHEAD by ~68
  unpushed commits (exact: `git rev-list --count origin/main..HEAD`):
  `633ee65b`+`dc3a0102` (v0.64.5 plan/refresh docs) + m276..m292 + m293 +
  m294 + m295 + m296 + m297 + m298 + m299 + m301..m309 + m310 + handoff follow-ups + 2 attempt logs (stdlib-6 leg b m291 reverted, see
  docs/failed_attempts.md; the bindings-4 m288 attempt is superseded by
  m292, and the stdlib-6 leg-b m295 attempt supersedes its failed entry).
  Do NOT push without the owner's authorization.
- v0.64.5 batches m276-m292 + m293..m299 + m301..m309 + m310 (commit -> scope; all relayed):
  - `0268c7d9` m276 env-unwrap veto on concretely-typed names (T1 kv AV).
  - `29e35ef8` m277 cast-call C handoff keeps the RAW fn code (two-word
    env: 0=thunk, 1=raw) + nested fn-type markers parse (T2 B-22 + 1403).
  - `484d9f21` m278 same-named types keep module identity (T3 2137).
  - `1a8141fb` m279 as_ptr/as_mut_ptr inference -- `*T + Int` was Str
    concat (B-17 + B-16).
  - `e1207485` m280 unsafe Copy-Out pointer gate + temp Vec[Str] concat
    (2230).
  - `77827766` m281 `p[i]` raw-pointer byte reads were the UTF-8
    `xiom_char_at` decoder (B-24 stale-zero rounds; B-20/B-11 readback);
    bare `null` at a pointer arg was a stack byte (B-23); `base + off`
    confined tails Str-copied (B-20 helper). Locks `ffi_ptr_slot_readback`
    + `null_literal_marshal`.
  - `d6690868` m282 fn-typed value `f as *UInt8` handed the closure ENV
    handle instead of raw code (thread/spawn worker-entry AV). Lock
    `ffi_fn_env_cast` + `m282_fn_env_raw_ptr`.
  - `3ef4d6a9` m283 (a) postcondition call-cycles -> per-function contract
    re-entrancy counters (1549-packages-8); (b) nested enum payloads
    resolve the concrete monomorph (B-26 1524-bindings-2). Locks
    `e2e_m283_contract_cycle` + `e2e_m283_nested_payload_read`.
  - `186edb3a` m284 `collect_ident_names` had no `Expr::MutRef` arm ->
    `&mut q` in a confined block was never captured (B-21 2140-bindings).
    Lock `e2e_m284_mut_of_struct_return`.
  - `77dfc786` m285 `type_arg_compatible` requires a BASE match when exactly
    one side is a parameterized container -- a concrete scalar is no longer
    treated as an erased container, so `&Vec[Float64]` for `&Vec[Vec[Float64]]`
    is now a call-site T001 (1549-stdlib-5 HIGH; witness compiled rc 0 then
    AVed 0xC0000005). Placeholder carve-outs (wildcard `_`, generic `T`/`*T`,
    empty/unit payloads) keep the legacy bare-vs-parameterized flows. Locks
    `m285_vec_shape_arg_mismatch` + `m285_vec_shape_arg_matched`
    (checker_locks.rs).
  - `7e07d144` m286 cross-type generic callbacks kept their substituted
    types: (1) bare/bracket generic-call bindings substitute the declared
    return via a per-CALL-SITE record (`bare_generic_call_return_xiom`;
    placeholder elems upgrade once the concrete xiom type is recorded);
    (2) match scrutinee payloads fall back to the substituted call return
    ("Option[Str]"); (3) method arg inference skips a leading `Self` param
    so `f: fn(T) -> U` resolves U from the argument's own signature (was
    recorded as the raw "U"); (4) Str `<`/`>`/`<=`/`>=` compare CONTENT via
    strcmp (was first-byte, so `Ord[Str].compare` said "1" == "10"). All
    three 1549-stdlib-7 repros green (map 41->0, core_map 41->0, sortbykey
    1->0; fnptr stays green). Locks `e2e_m286_cross_type_callback_bind` /
    `_core_map_match_payload` / `_sort_by_key_cross_type` /
    `_str_lexicographic_compare` (ci group 2).
  - `1f01e812` m287 `use xiom.convert.tostring` pulled
    `math.mathematical_logic` in; the adapter closure-FIELD call
    `self.predicate(&v)` (FilterIter/TakeWhileIter.next) was hijacked by the
    same-leaf free fn via the UFCS/suffix scans (a DIRECT call bypassing the
    closure env; predicates never saw values). Instance fn-typed fields now
    skip those scans so the env-first closure call wins (1548-stdlib-3 HIGH).
    Lock `e2e_m287_tostring_import_adapters` (ci group 2).
  - `554489f5` m288 LOCK for 1548-stdlib-4 (`x.a@pre` on a `&mut` param
    field): the filed witness is RED on the archived v0.64.4 binary and
    GREEN at HEAD -- fixed within the m276-m287 range (clause/entry-snapshot
    machinery); enforcement verified LIVE at HEAD (+5 mutated body still
    violates). Lock `e2e_m288_mut_param_field_pre` (ci group 2).
  - NOTE: 1548-bindings-4 has NO m288 fix (attempt reverted; see
    docs/failed_attempts.md 2026-10-13 and the bus) -- next free batch is
    m290.
  - `b33a8439` m289 annotated `Slice[Int]` locals allocated an i64 slot
    holding field 0 (the data pointer): `concrete_type_for` fell through to
    `type_from_ast`, which strips Slice to its element ("Int" -> i64).
    `.len()` then strlen'd the data buffer (1 instead of 3) and the slice
    helpers AV'd. Now maps `Type::Slice(_)` to "Slice" -> canonical
    `%struct.Slice` (1548-stdlib-5; its two filed witness FILES were stale
    green on v0.64.4 -- the extended list was the live part). Lock
    `e2e_m289_slice_annotated_local` (ci group 2).
  - `bc716bca` m290 PARTIAL for 1548-stdlib-6: leg a FIXED -- `get_type`
    expands a dotted name's first segment via
    use_alias_paths/local_module_paths, so `ser.SerializeError{..}` resolves
    (was T001). Leg b OPEN -- `v.json_get_path(&path)` on a by-value
    JsonValue still C001 `'JsonValue.json_get_path'`; the UFCS candidate
    helper requires a pointer-like receiver (see COMPILER_BUGS m290).
    Lock `e2e_m290_alias_type_path` (ci group 2).
  - `a6bfe1e1` m291 **ENFORCEMENT GATE FIX + two items**: (1) m283's
    contract re-entrancy guard had made runtime requires/ensures
    enforcement SILENTLY DEAD (skip test `d > 0` with the depth counter
    incremented at function entry, d==1 at the first clause site); test is
    now `d > 1` (only re-entrant evaluations skip). (2) the guard form
    `(result.is_ok == true) => ...` now rebinds the payload like
    `result is Ok =>` (1548-stdlib-7). (3) 1548-stdlib-4 CORRECTION: its
    m288 green was MASKED by (1); the real fix is the Paren arm in
    `collect_atpre_vars` (`(x.a@pre < 100) => ...` collected no snapshot
    vars). Witness rcs 1->0 with enforcement LIVE. Locks
    `e2e_m291_ensures_isok_guard` + checker_locks
    `m291_contract_enforcement_live` (negative twin); `e2e_m288_mut_param_
    field_pre` now passes for real. LESSON: any item closed by a green
    contract BEFORE m291 must be re-trusted only with enforcement live.
  - `c5fe4d80` m292 1548-bindings-4 FIXED: the catalog pre-load worklist
    skipped `[xiom,ffi]` whenever the BARE leaf "ffi" was already in
    `self.modules`; `flatten_submodules` registers nested program modules
    under their leaf, so `module probe.ffi` + `use xiom.ffi;` bound the
    program's own module and the stdlib submodule fns (xiom.ffi.c) never
    registered (T001 undefined variable; this is why m288's direct-load
    registration patches had nothing persistent to attach to). Guard is
    now single-segment-only (`prefix.len() == 1 && ...`). Locks
    `e2e_m292_ffi_alias_shadow` + fixture (negative A/B via a rebuilt
    pre-fix binary).
  - `5fbd54be` m295 1548-stdlib-6 leg b FIXED (item fully done; leg a
    m290): `v.json_get_path(&path)` on a by-value JsonValue was C001
    'JsonValue.json_get_path' -- `ufcs_registered_fn_for_receiver` required
    a pointer-like receiver, so the registered fn (param 0
    `%struct.JsonValue` BY VALUE) never matched. Codegen now mirrors the
    checker's param-0 UFCS search for bound `Ident` receivers
    (xiom-check lib.rs ~8786). IR evidence:
    `call %struct.Option__JsonValue @serialize.json.json_get_path(...)`.
    Locks `e2e_m295_foreign_method_call` (runtime payload check) + m290
    leg-a stays green. NOTE: the m291 "no .json_get_path key" trace claim
    was wrong (both same-leaf keys are registered) -- corrected in
    COMPILER_BUGS m295.
  - `c18e550e` m298 same-named types keep module identity in two more
    paths (1956-orbitdb HIGH + 1709-xvector): (a) the R44 all-catalog
    facade exemption compared shape STRINGS, so the engine's BTree stayed
    on the bare key while its `store` refs had been rewritten to the
    package's NodeStore -> `payloads.push/pop` C001; the triage now marks a
    group conflicting when any shape references ANOTHER multi-owner leaf
    (`shape_mentions_leaves`). (b) `build_renames` rewrote only
    dotted/aliased spellings, so a bare same-leaf field ref stayed bare and
    the suffix pick took the shortest key; a bare ref now binds the UNIQUE
    imported owner (ambiguous/local keeps legacy). Locks
    `e2e_m298_orbitdb_compose` + `e2e_m298_xvector_counter_shadow` (A/B
    negative twins on a rebuilt pre-fix binary). NOTE: the orbitdb probe
    needs the stdlib LANE tree for its byte-IO primitives (bundle dev-env);
    the vendored pin dies earlier on unrelated T001s.
  - `2914cb15` m293 Map/Set deep `==` for by-value fields (packages-11):
    `is_deep_eq_container` excluded Map/Set, so `m1 == m2` hit the
    struct-equality cascade and emitted `icmp eq %struct.Vec` (clang
    reject). deep_eq.rs now gates Map/Set/BTreeMap/BTreeSet (concrete
    K/V/T args, strcmp for Str keys) + legacy non-generic HashSet;
    expr.rs recursion for by-value container fields without `.eq`.
    Locks `e2e_m293_mapset_eq`. CAVEATS recorded (COMPILER_BUGS m293):
    same-order layout semantics (permuted lookup follow-up), HashMap[K,V]
    Str keys unverified, and a latent pre-existing m239 gap: `Vec[Bool]`
    compares with an i64 stride (legacy mapch.HashMap == can
    false-negative).
  - `f6690a77` m294 bare `&mut`/`&` reads load the pointee in EVERY value
    sink (packages-9 HIGH): m197 only covered `return p` + arithmetic;
    let/var bindings returned the stack ADDRESS, `v[p]` AV'd, casts/negation
    read pointer bits, and Eq/Neq double-derefed (`inttoptr` of the loaded
    value -> AV). autoderef now runs at let/var + assign/deref-store + Unary
    + scalar `As`; new `compile_index_i64` at all 12 index sites; Eq/Neq
    excluded from the m197 binop deref. Locks `e2e_m294_mut_int_bare`
    (+ m197/m284 stay green). Caveat: i64-address `&T`/`&Str` params stay
    outside the pointer-typed gate (unchanged).
  - `ee4fc29a` m296 2320-bindings part 1 FIXED: variants of `Vec` sized
    buffers under the documented 8-bytes-per-slot contract -- the stdlib
    `with_len` zero-fills n * 8 but construction sized `cap * elem_size`
    (Vec[UInt8] 16 bytes for 16 slots), so `with_len(16)` wrote 128 bytes
    past a 16-byte region (STATUS_HEAP_CORRUPTION class in xiom.shaderc).
    All construction sites now floor slots at 8 bytes via `vec_slot_size`;
    elements > 8 bytes keep their size; push growth element-sized. IR
    twin: pre `malloc(i64 16)` -> post `malloc(i64 128)`. Lock
    `e2e_m296_vec_with_len`. Parts 2/3 remain open.
  - `6797197d` m297 1615-stdlib FIXED (HIGH): `Path.is_dir`'s body
    bare-call of the same-leaf free fn `is_dir(self.inner)` self-bound
    (legacy in_caller_module fallback + G-10 implicit-self override) ->
    `@Path.is_dir` recursion -> depth-500 llvm.trap -> 0xC000001D for any
    program touching it. Fix mirrors the m261 self-exclusion and makes
    G-10 skip only when the standard resolution bound a DIFFERENT real
    fn. Lock `e2e_m297_path_println_context` + i2/hot-reload regressions
    stay green.
  - `50361abf` m299 1549-packages-12 FIXED: `let (k, v) = &vec[i]` bound
    component ADDRESSES (m209 `&T`-annotated i64 refs); value uses leaked
    addresses and the Str concat path AV'd. The arm now loads VALUE
    copies (m209 `k == &key` compare still works); the
    `regress_m209_tuple_ref_destructure` test was updated to the
    value-copy shape. Lock `e2e_m299_tuple_ref_destructure`.
  - `1b78d20f` m301 1549-stdlib-4 FIXED: clause-position `result[i]` never
    registered the synthetic `result` in `local_vec_elem`, so it fell to
    the scalar `emit_elem_load` fallback -- `Vec[Float64]` f64 bits were
    sitofp'd (1.0 -> 4.6e18 -> false `contract violated: ensures`) and
    `Vec[Vec[T]]` read the inner data pointer as a Vec header. The
    witness's [1,2] Int control passed only by COINCIDENCE (the [7,9] row
    violates pre-fix). contracts.rs now registers the return Vec element
    per clause + snapshots the vec maps. Lock
    `e2e_m301_clause_float_vec_index` (FAILS Some(1) on a pre-fix binary).
    ALSO m301: 1549-packages-2 (io fs_read ensures) = NO-REPRO on HEAD --
    73 runs over 7 multi-module shapes (sibling/chain/permuted imports,
    --parallel-codegen, package-catalog kv shape, heavy append loops,
    stdlib smoke_io2) all green with enforcement LIVE; item stays open
    pending a faithful repro (original nondeterminism was v0.64.0/2-era
    clause machinery that m291 rewrote).
  - `f67adcea` m303 stdlib-2 + stdlib-3 FIXED (one batch for the m300
    fixer's two items): (a) `geometry_3d.Box{..}` T001 unknown-type and
    the spelling-compare param check made aabb_contains/
    aabb_intersection uncallable -- xiom-check now resolves
    module-qualified catalog type spellings in signatures+calls, codegen
    carries them through construction/param matching; (b)
    `var l2 = lu.0;` on an un-annotated nested Vec[Vec[Float64]] result
    lost a level (`type_string_full` erased tuple element containers to
    "Vec" -> scalar elem path, row len 0) -- container args kept in tuple
    types + tuple-field vec reads. Evidence re-run here:
    p_geom_box_unnameable v0.64.4 compile rc 1 (T001 at 31:11) -> rc 0 +
    run rc 0; p_geom_matrix_result_infer v0.64.4 run rc 4 -> rc 0. Locks
    e2e_m300_geom_box_unnameable + e2e_m300_geom_matrix_result_infer.
  - `8c298c91` m302 1659-orbitdb FIXED: a Vec bound/assigned from an
    ALIASING RHS (`var b = a;`, `var w = arena[i];`, `var k = s.keys;`,
    `b = a;`) now deep-copies the buffer at the binding
    (`emit_vec_value_deep_copy`, cap=max(len,1) to dodge the
    realloc(p,0) NULL trap in the stdlib extract-push-storeback idiom);
    fresh values stay uncopied; push/pop unchanged. Witness
    `src[0]=99 copy[0]=99` -> `src[0]=1 copy[0]=99`; fixture pre-fix exit
    4 -> 0. CAVEATS (recorded): by-value call args, `push()` args,
    for-loop element bindings, and whole-struct copies with Vec FIELDS
    still share buffers; generic-boxed container-field handles outside
    the predicate.
  - `d96e9a2b` m304 1550-orbitdb-2 FIXED: standalone `--check` of a
    self-referential module failed T001 `expected Engine, found module`
    while the same file compiled clean AS AN IMPORTED module -- the
    declared chain's segments are flattened into the module map and
    `check_module_field_access` tried module-qualified resolution before
    local-scope lookup, so `db.engine` on a `db` parameter resolved to the
    submodule. Locals now shadow module aliases (locals-first precedence),
    mirroring the Ident arm. Lock `e2e_m304_module_local_shadow`.
  - `81e4743c` m305 1219-orbitdb PART A FIXED (PART B was m260 -- item
    fully done): a tuple from an enum CONSTRUCTOR call
    (`Ok((Color.Custom(n), n + 3))`) fell back to the i64-erased element
    type and built a 16-byte `Tuple__Int__Int` (tag in slot 0, payload
    dropped) while the read side used the 24-byte `Tuple__Color__Int` --
    silent wrong values + 0xC000001D on the ub-grade read. Enum ctor-call
    elements are now named by the parent enum key (expr.rs literal path +
    `infer_llvm_type` tuple arm). Lock `e2e_m305_enum_ctor_tuple`.
  - `06860b27` m306 registry/pkg cluster FIXED (four tooling items):
    (a) 1549-registry-2 trust hint now pins the registry INDEX key
    (--index-key, C5 signed index digest) instead of the ephemeral OIDC
    publisher key; (b) 1549-registry `xiom pkg yank <pkg>@<ver>` against
    the live contract (Bearer token, optional reason, ok=true required,
    error JSON surfaced); (c) 1549-registry-3 `xiom pkg resolve
    <name>@<spec>` outside a workspace (index.json, no auth, closure +
    --json); (d) 1548-registry-2 publish pack guard (build/artifact
    refusal list + gitignore-syntax .xiomignore + declared-modules keep
    set; refuse writes NOTHING; --force reports). pkg suite 92/92.
  - `7822f4b6` m307 1324-selfhost FIXED: a variant-pattern binding NAME
    won the direct field lookup before the canonical field at position fi
    (NkExprField(recv, name_node) read slot 0 = 461 for a parsed 23;
    same class as the 2026-10-02 large-dispatch finding). Canonical field
    at fi now wins; name match is fallback only. Red on v0.64.4 too.
    Locks `e2e_m307_variant_binding_shadow` (+ `_ir` twin: read_renamed
    must GEP `, i32 0, i32 2`, never `, i32 0, i32 1`).
  - `d52a813f` m308 playground/benchmark cluster FIXED (three items):
    (a) 1553-playground-2 by-ref Str comparator params (`&Str`/`&mut Str`)
    fell to first-byte pointer compare (the m286 strcmp branch only matched
    plain "Str"); ref spellings now recognized, find_min returns 2 not 0;
    (b) 1554-playground stdout `fflush(NULL)` before blocking C text-stdio
    reads (pipe block-buffering delayed prompts to exit; playwright probe
    3002/3004 ms); (c) 1554-benchmark guard-page arm/disarm elided for
    arena-free confined blocks (ONE VirtualProtect/mprotect per block entry
    was the ~220 ms t3 floor; ablation 130 -> 2 ms; fault containment
    lock kept). 7 locks incl. IR twins.
  - `776f8c9d` m310 packages cluster FIXED (two items): (a)
    1549-packages-7 local-fallback installs place files + `[dependencies]`
    RANGE specs resolve (old resolver used literal starts_with; new
    xiom-graph version.rs constraint matching); (b) 1549-packages-5 a
    dependency's shipped port.args.json (${PACKAGE_DIR}) applies at
    CONSUMER build time (no manual --c-source; was `undefined symbol`
    on v0.64.4). Battery: pkg 95 + 1, graph 40, e2e g2 209.
  - `d1eab753` m309 2320-bindings parts 2 + 3 FIXED (item fully fixed with
    part 1 m296): (a) confined-block Vec Copy-Out v1 -- a Vec built inside
    a confined block routed its buffer to the guard arena and faulted the
    caller after `xiom_guard_heap_exit` (0xC0000005; the xiom.shaderc
    trampolined crash class); `%struct.Vec` tails on BOTH lowering paths
    (stmt.rs return-inside-block + expr.rs block tail) now malloc+memcpy
    to the main heap before the arena reset; shaderc passes 17/17 x10 with
    all 50 `#[unsafe_direct]` markers stripped; (b) `register_external_
    module` merged export maps one level deep so 4-segment sibling modules
    dropped each other's subtree (LIFO last-use won, import-order flip);
    SubModule collisions now recurse via
    `merge_module_exports_first_wins`. Locks
    `e2e_m309_confined_vec_copy_out` (+IR twin) +
    `e2e_m309_four_segment_const_import` (both orders).
- CLOSED / awaiting reporter re-verify at the next archive (compiler work
  done; items are `fixed` on the bus): T1-T3, B-17+B-16, B-23+B-24
  (2051-bindings-2), B-11+B-20 readback family (0304-bindings +
  1548-bindings-2), 1549-packages-3, 1549-stdlib-5 (shape-mismatched &Vec
  arg is now a call-site T001), 1549-stdlib-7 (cross-type generic
  callbacks), 1548-stdlib-3 (tostring import no longer hijacks adapter
  closure fields), 1548-bindings-4 (leaf-colliding nested module + stdlib
  import -- m292), 1549-packages-11 (Map/Set == -- m293), 1549-packages-9
  (bare &mut/& value sinks -- m294), 1615-stdlib (Path.is_dir self-bind
  SIGILL -- m297), 1549-packages-12 (tuple-ref destructure value copies --
  m299), 1549-stdlib-4 (clause result[i] element binding -- m301),
  1549-stdlib-2 + 1549-stdlib-3 (module-qualified type spellings + tuple
  container args -- m303), 1659-orbitdb (Vec copy-alias deep copy -- m302),
  1550-orbitdb-2 (locals shadow module chain segments -- m304),
  1219-orbitdb (enum ctor-call tuple shape -- m305 PART A + m260 PART B),
  1548-registry-2 + 1549-registry/2/3 (registry cluster -- m306),
  1324-selfhost (variant-pattern canonical field -- m307),
  1553-playground-2 + 1554-playground + 1554-benchmark (playground cluster
  -- m308), 1549-packages-5 + 1549-packages-7 (install/build cluster --
  m310),
  1548-stdlib-6 (leg a m290 alias type paths + leg b m295
  by-value UFCS), 1956-orbitdb + 1709-xvector (same-named types keep
  module identity -- m298), 1548-stdlib-4 (@pre on &mut param fields snapshots the
  entry value -- REAL fix m291 Paren arm; the m288 green was masked), 1548-stdlib-7
  (guard-form ensures + enforcement restored), 1548-stdlib-5 (annotated
  Slice locals), B-25 (1309-bindings, re-probe GREEN),
  B-26 (1524-bindings-2), B-21 (2140-bindings), 1549-packages-8 (contract
  cycles), 1549-stdlib-6 (VERIFIED FIXED cross-platform: clause re-added in
  a scratch tree, stdlib repro + family probe + smoke suite green on
  Windows AND WSL/Ubuntu), 1358-orbitdb (NOT a compiler bug: the documented
  Linux artifact exits 1 for both probes in 20+ variants; rc=0 reproduces
  with the sibling `xiom-dbg` DAP adapter binary -- the lane harness ran
  the wrong binary; asked them to pin/assert XIOM_COMPILER).
- BLOCKED ON THE STDLIB LANE (compiler-lane items filed with ready diffs):
  - `REL-20261013-1423-compiler` -- `xiom.thread.spawn` result buffer is
    guard-arena memory freed at block exit before join (UAF). Thread gate
    (2053/1627/1402) stays open until the buffer is hoisted to main-heap
    allocation outside the confined block.
  - `REL-20261013-1445-compiler` -- arena-aware `xiom_guard_free` +
    `ffi.free` route (alloc-guard-spin 1548-bindings-5). Verified in a
    scratch tree; applies at the next stdlib pin.
- OPEN FINDINGS recorded (no fix yet; in COMPILER_BUGS):
  - (-O2 constant-fold pruning of contract checks: FIXED m311 `3237aaa1`
    -- volatile i8 round-trip barrier in `compile_contract_check`; lock
    `o2_contract_barrier_cli` 4/4.)
  - Duplicate stdlib trees + cross-tree function-body mixing in ad-hoc
    compiles (dedupe module registration by canonical root).
  - Vec has no drop glue (2032-engine) -- language memory-model decision,
    owner-level.
- REMAINING CANDIDATE QUEUE (reds; the bus + the check file are the
  record). Priority order for the next session:
  1. borrow/codegen: 1548-bindings-4 (m292), 1548-stdlib-6 (m295),
     1549-packages-11 (m293), 1549-packages-12 (m299), 1549-stdlib-2 +
     1549-stdlib-3 (m303), 1549-stdlib-4 (m301) DONE. OPEN:
     1549-packages-2 -- NO-REPRO m301 (73 runs / 7 multi-module shapes
     green with enforcement live; needs a faithful repro).
     (1549-packages-10 is a wave stale-green close.)
   2. resolver/checker: 1549-packages-9 DONE (m294), 1550-orbitdb-2 DONE
      (m304); 1956-orbitdb + 1709-xvector DONE (m298).
  3. allocator/runtime: 2320-bindings DONE (part 1 m296 + parts 2/3 m309);
     2032 (owner).
  4. contracts/clauses: 1549-stdlib-4 DONE (m301); 1549-packages-2
     NO-REPRO on HEAD (m301, stays open); 1548-stdlib-7.
   5. runtime/misc: 1615-stdlib DONE (m297); 1659-orbitdb DONE (m302;
      call-arg/struct-copy shapes remain shared -- see COMPILER_BUGS
      m302), 1219-orbitdb DONE (PART A m305 + PART B m260), 1324-selfhost
      DONE (m307), 1553-playground-2 + 1554-playground + 1554-benchmark
      DONE (m308).
  6. tooling/registry: 1548-registry-2 + 1549-registry/2/3 DONE (m306);
     1549-packages-5/7 DONE (m310).
   7. macOS: 1549-pulse + 1609-pulse (typed `@llvm.memset`) -- needs a NEW
      archive to verify.
   8. ENGINE RELAY BATCH -- 17 items queued into the candidate record on
      2026-10-14 (rows added; no triage yet): 1819 (Int ptr arg, B-16
      class), 2035 (&Str param spelling), 2042 + 2042-2 (Int128/UInt128
      miscompile HIGH; UInt64 bit-63), 2119 (large-main startup AV, low),
      2139 (enum-variant/geom name collision), 2215 + 2215-2 (Vec push in
      match arm; move-semantics pin), 2227 (UInt64 div/mod sign), 2238 x3
      (CLI lane: vecvecstr / fnvalue_records / meshencoder), 2256 (const
      shadowing hang + pub use drops consts), 2324-2 (var fn export drop),
      2324-3 (diamond pub const), 1158 (ident named spawn drops fn),
      1213 (layout-sensitive probe AV, low). These are the next fixer-wave
      candidates after m311-m313 land.
   IN FLIGHT / NEXT SESSION'S FIRST ACTIONS (Agent Manager fixer worktrees;
   details in section 8 item 0): m313 fix/m313-ffi-free-guard LANDED on main
   `56234edf` (ffi.free call-site lowering; acceptance GREEN on stdlib main
   -- spincheck 3/3 `v=7` exit 0 <=263 ms; compiler note filed on
   REL-20261013-1445-compiler); m312 fix/m312-thread-spawn-readback LANDED
   on main `bc1d821b` (ptr-typed struct-field READ PATH; locks + IR twin
   green) -- acceptance vs stdlib main NOT green: residual root-caused to
   the RUNTIME (xiom_task_trampoline stores the i64 result as C `long` = 4
   bytes on Windows; low32 == 42 in every garbage readback; fix =
   long long/int64_t in the trampoline ~5962-5977, filed on
   REL-20261013-1423; thread gate 2053/1627/1402 stays OPEN, do NOT flip
   the spawn e2e without it); m311 fix/m311-o2-no-fold-barrier LANDED on
   main `066bc253` (fixer commit 3237aaa1 + doc/ci tokens; contract no-fold
   barrier via volatile round-trip; lock 4/4 + battery green). ALL THREE
   INTEGRATED + relayed (bus 071f5aa/8dbc424/d7c26ba). ENGINE WAVE m314-m317
   ALL LANDED on main + relayed (bus c036f0a/6f88343): m314 `b7ed15b1`
   (2042 + -2042-engine-2 + 2227: Int128/UInt128 16-byte slots/align-16,
   16-byte memcpy stores/typed loads, zext widening, icmp u*/udiv/urem),
   m315 `32923053` (2139: current-module variant precedence + struct-fit
   literal), m316 `f4583fb1` (1158: parser soft-keyword struct-literal
   lookahead -- NOT a catalog shadow), m317 `af3d6460` (2256 + 2324-2 +
   2324-3: const-substitution cycle guard/hang, cross-module const + export
   surfaces, pub-use consts, pub var). WAVE 2: m318 (2215 + 2215-2) and
   m319 (2238 x3 CLI lane) still in flight; m320 `b672a19f` LANDED
   (1819 `ptr as Int` at named-call boundaries + 2035 `&Str` param
   spelling; locks x4; relay 0817396). m321 (2119 + 1213) IN FLIGHT:
   2119's bisect_88 crash is a WRONG-EXPECTATION reduction (chord cannot
   fire -> unguarded events[0] on an empty Vec; identical on the pinned
   v0.64.4 toolchain; the real engine probe is 22/22) PLUS a real gap --
   no Vec read-path bounds lock (default silent UB read, --overflow-checks
   traps) -- being pinned; 1213 REPRODUCES (compile rc 0, run 0xC0000005,
   crash at line 381, strcmp-on-garbage class) and is being localized.
   THREAD GATE GREEN pre-pin: stdlib main `ae9c963` (wave 139,
   long->long-long trampoline) -- dbg131s/dbgworker x5/5 `joined 42`
   (independent re-run 2026-10-14T17:33Z). PIN REQUEST (prep) filed to
   stdlib: REL-20261014-1739-compiler (bus af92693) -- freeze a sha
   carrying wave 139, tag stdlib-v0.64.5 on my signal. OWNER GO for
   v0.64.5 release received (no supervision; pushes authorized for the
   release leg). Next free m322.
   RELEASE v0.64.5 PUNCH LIST (prepared 2026-10-14T18:35Z):
   - freeze = m321 resolved + stdlib tag reply on REL-20261014-1739-compiler
     (tag on my signal after the release-wave push; also ask them to keep
     their release-notes/v0.64.5.md fragment in the tag for the converter).
   - version: Cargo.toml `[workspace.package]` 0.64.4 -> 0.64.5;
     selfhost/src/codegen.xi `SELFHOST_VERSION` -> 0.64.5; README pin text.
   - pin: `STDLIB_VERSION` (repo root, read by crates/xiom/build.rs) -> tag
     sha; vendored `stdlib/` worktree refreshed to the tag
     (scripts/fetch-stdlib.ps1).
   - notes: release-notes/v0.64.5.md (Summary + Highlights `kind:` lines;
     max 6 merged with the stdlib fragment) then
     `xiom-release-notes convert --tag v0.64.5` (converts + validates sync).
   - record: docs/RELEASE_GATE_v0.64.5.md modeled on v0.64.4 (preconditions
     / Gate A suites / blockers / Gate D tag+publish).
   - battery: full e2e (whole ci.yml list), feature-reg 549, check 199 /
     parser / codegen 12 / graph 36 / verify / pkg / release-notes 6 /
     stdlib_tests, front-locks, nine-tool release build (all report
     v0.64.5), ascii guard, doctor; thread acceptance at the PIN.
   - publish: push main -> tag `v0.64.5` (guard: tag == workspace version,
     tag on main) -> release.yml; verify assets + SHA256SUMS; VSIX is
     toolchain-only (editors/vscode/package.json stays 0.12.2).
   RELEASE LEG STATUS (2026-10-14T19:50Z): ALL queue items resolved
   (wave 2 + m321 landed; engine batch fully closed). Release-wave PUSHED
   to origin at `1e4a444e` (90 commits from 91860f11; workspace + selfhost
   bumped to v0.64.5; VSIX README pin). Dry-run archive run 38626737822 on
   that sha (in progress). Tag signal sent to stdlib (bus 06d0a7c) --
   waiting for the annotated stdlib-v0.64.5 tag ref + sha reply on
   REL-20261014-1739-compiler (frozen candidate db2fbe8; asked to drop the
   now-stale alloc/free known-issue line). NEXT: pin commit (STDLIB_VERSION
   + vendored refresh + notes JSON convert + gate record) + full battery
   incl. thread acceptance at the pin -> push -> tag v0.64.5 -> release
   run -> verify assets -> record. Origin CI (ci.yml) also running on the
   push.
   RELEASE LEG STATUS (2026-10-15T09:20Z): NOT released yet; stdlib tag
   HELD pending the candidate battery. (a) m322 `eb256cf3` LANDED: the
   gate's full-suite run found + fixed three latent regressions (signed
   cast targets decide compares -- m32_int_0194/0195; scalar-pointee
   field indexing -- m37_bug45; full e2e 2544 passed / 0 failed / 4
   ignored; the three tests now in the ci.yml e2e list). (b) stdlib's
   candidate checks bounced FOUR classes on the dry-run archive
   (REL-20261014-2123-stdlib, acked): Cell[Int8] set/get readback (m323),
   Result Ok-pair Vec lengths 0xFFFFFFFF + p_wave107/108 (m324),
   zext-float cast p_sweep_single_param (m325), duplicate
   LinkedList.invariant_check p_wave7/8 (m326) -- all four REPRODUCED on
   current main; fixers m323-m326 in flight. Stdlib tag target is now
   `c8d713a` (their notes-cleanup commit; no other waves ride).
   NEXT: integrate m323-m326 (standard flow), push, dispatch a FRESH
   dry-run archive, stdlib re-runs the candidate battery -> on green:
   tag signal -> pin commit (STDLIB_VERSION -> tag sha) + notes JSON +
   gate record + full battery incl. thread acceptance at the pin ->
   push -> tag v0.64.5 -> release run -> verify assets -> record.
   UPDATE 2026-10-15T10:05Z: THREE OF FOUR classes landed on main --
   m323 `0cde8579` (Cell[Int8] set/get: skip the self slot pairing generic
   method decl params; witness rc 2->0), m324 `9914c2ed` (Ok-pair Vec
   lengths: tuple struct keys canonicalized at the first bracket;
   witnesses rc 3->0, p_wave107 20->0, p_wave108 32->0, aes-gcm AV->0),
   m325 `6caf7a16` (float operands never take integer extensions; witness
   p_sweep_single_param compilefail->green); all with locks + ci tokens;
   battery g1 16 + g2 239 + codegen 12 green over the stack; relay note
   filed. m326 (duplicate LinkedList.invariant_check) IN FLIGHT: checker
   worklist dedupe + codegen emit-once guard implemented, p_wave7 green,
   p_wave8 compiles but traps at run (being localized; files touched
   include xiom-check/lib.rs, codegen context/contracts/decl/expr/lib,
   xiom/src/lib.rs). ON m326 LANDING: full e2e (2544+ baseline) -> push ->
   fresh dry-run -> stdlib candidate re-run -> tag signal.
   UPDATE 2026-10-15T10:50Z: m326 LANDED `1f7c1c22` -- ALL FOUR classes
   integrated (m323 `0cde8579`, m324 `9914c2ed`, m325 `6caf7a16`, m326
   `1f7c1c22`; the wave8 trap was the @pre snapshot deep-copy reading the
   6-field list as a Vec header -> malloc(huge) -> trap). p_wave7/8 probes
   compile+run 0/0; stack battery green (g1 16, g2 241, codegen 12, check
   serial 199, front-locks). 2123 relayed fixed. FULL E2E RUNNING (bgp
   _13f333cf6001X2XAgtJuKKWNsd); ON GREEN: push -> fresh dry-run ->
   archive signal to stdlib (REL-20261014-1739-compiler) -> candidate
   re-run -> tag -> pin -> publish.
   NEXT-CANDIDATE QUEUE (owner decision 2026-10-15T11:26Z): GENERAL
   Darwin/macOS support -- emit the matching declare for the typed
   `@llvm.memset.p0i8.i64` (sites emitter.rs:1153, expr.rs:4229,
   stmt.rs:737/1190; or switch to the untyped form) + the darwin runtime
   items (tf x64 symbols, `_SC_AVPHYS_PAGES`, fp128 arm64 asm; stdlib
   runtime, cross-filed) -- so the ENGINE lane can build and run macOS
   games. v0.64.5 DOES NOT wait for it (owner: hurry the release; lots of
   relays depend on it); pulse items REL-20261010-1549-pulse +
   REL-20261010-1609-pulse stay acked with the decision noted (bus
   39bfb97); v0.64.5 still produces + job-verifies both macOS legs.
   UPDATE 2026-10-15T11:30Z: FULL E2E = 2549 passed / 1 FAILED / 4
   ignored: e2e_m37_bug55_unsafe_ptr_capture. Bisect: green at m322/m323/
   m324/m325, RED at m326 (`1f7c1c22`) -- your m326 llvm_type_for
   qualification reroutes an xiom.rc field write into the struct-value
   alloca path and DROPS the store-back (IR A/B on @__unsafe_block_3:
   m325 stores `+1` to field1; m326 computes it, discards it + a dead
   load) -> the strong-count bump never lands -> upgraded Rc reads a
   freed box -> `up.get() != 100` (rc 2). Follow-up fix requested from
   the m326 session (ses_ec12801c2ffepB5lk7uSZ6UxEv); fix lands as an
   m327 batch (second commit on fix/m326...; cherry-pick the follow-up
   only), then FULL e2e again, then the push -> fresh dry-run -> stdlib
   archive signal. NOTHING pushes until bug55 is green.
   UPDATE 2026-10-15T11:40Z: m326 follow-up LANDED + integrated as m327
   `7a574f9c` (struct_field_names_for: types exact/leaf + type_meta
   exact/suffix; direct gep+store kept on the qualified struct path).
   bug55 0/0; m326 locks + p_wave7/8 probes 0/0; g1 16 + g2 241, codegen
   12, check serial 199. FULL e2e re-gate RUNNING (bgp
   _13f5c8479001l23yrQp4W3MQxd); ON GREEN: fetch + push main (~13
   commits) -> fresh dry-run -> signal stdlib (REL-20261014-1739-compiler)
   -> candidate battery -> tag -> pin -> publish. m327 note filed (relay).
   UPDATE 2026-10-15T12:00Z: FULL E2E GREEN 2550 passed / 0 failed / 4
   ignored -- stack FROZEN and PUSHED (`1e4a444e..13081238`). Fresh
   dry-run `38731274663` on `13081238` (in progress). Archive signal sent
   to stdlib (bus c836588): candidate battery re-run on the new archive;
   cut stdlib-v0.64.5 on c8d713a (or newer) + reply ref+sha on green.
   NEXT: tag reply -> pin commit (STDLIB_VERSION -> tag sha + vendored
   refresh + notes JSON convert + gate record finalize) + full battery
   incl. thread acceptance AT THE PIN -> push -> tag `v0.64.5` + push tag
   -> release run -> verify assets + SHA256SUMS -> record in the
   candidate record + handoff.
   UPDATE 2026-10-15T17:15Z (recovery after the loop stalled post-12:44Z,
   plus the pin-PR CI findings):
   - stdlib RE-CUT the tag: stdlib-v0.64.5 = tag obj 1667b3a8 -> commit
     `092dfa0` (old e082623 WITHDRAWN; their smoke_os_sync tripped the
     m296 pin-era Vec-ctor class; stdlib test fix only). xiom-std 0.64.5
     registry publish RESOLVED. Local STDLIB_VERSION now `092dfa0` +
     vendored worktree at the re-cut tag.
   - Pin PR xiom#6 (chore/pin-stdlib-v0.64.5, owner-opened; branch =
     main@13081238 + STDLIB_VERSION) CI found FOUR blockers:
     1) perf_budget_fmt_peek (windows leg): IR 113323 > 95000 --
        compiler-side (repros with BOTH pins); fixer m328
        (`ses_ebf5a8d7cffeqaPfwUpkg1JKnz`) running.
     2) stdlib_api_freeze_no_removals (ubuntu): 30 signatures missing --
        extractor regression, NOT stdlib content (repros with old pin);
        fixer m329 (`ses_ebf5a2ab9ffenEU0bfTVy7OH9c`) running; do NOT
        update the snapshot.
     3) ubuntu `dep_port_args` test: `C:/` is not absolute on POSIX ->
        double-prefixed dir; FIXED m330 `7628fbc0`.
     4) cargo-deny advisories RUSTSEC-2026-0346..0349 (rustls 0.23.45 ->
        `>=0.23.46`); FIXED in m330 (rustls 0.23.46).
     m330 pushed to main (`13081238..7628fbc0`).
   - Local uncommitted pin set (rides the PR branch): STDLIB_VERSION +
     notes JSON (6 highlights; convert+verify green) + gate record;
     thread acceptance at the pin 5/5+5/5; pin e2e 2550/0/4.
   - m331 `b6cf620d` LANDED + pushed (owner-approved quick Darwin fix):
     `@llvm.memset.p0i8.i64` declared in the builtin preamble (4 sites)
     + `hardcoded_declare_names`; PULSE 1549/1609 items acked-updated
     (bus 4152229); proper darwin work queued v0.64.6. NOTE: the declare
     adds ~49 bytes to every module's preamble -- when m328's budget
     lands, re-run perf_budget_tests after BOTH are on main (m328 was
     calibrated without the declare).
   - NEXT: m328+m329 land -> push main -> rebuild the PR branch
     (main + pin set) -> PR CI green -> merge xiom#6 -> tag v0.64.5 ->
     release run -> verify assets -> record -> goal_report complete.
   UPDATE 2026-10-15T18:35Z: ALL pin-CI blockers cleared on main --
   m328 `738e7158` (perf: leaner guards + 95k->112k + rationale),
   m329 `989c1bc9` (freeze: wave-105 canonicalization + wave-98 M7
   snapshot repair), m330 `7628fbc0` (POSIX test + rustls 0.23.46),
   m331 `b6cf620d` (memset declare). Pin-gate suites at the pin state:
   e2e 2550/0/4, feature-reg 549, workspace libs, fuzz 24, robustness 63,
   perf 3/3, freeze 3/3, stdlib 40 + execution 85, front-locks, ascii,
   thread acceptance 5/5+5/5. Pin branch `chore/pin-stdlib-v0.64.5`
   FORCE-PUSHED with the full pin set (`858d2e8d` = main + STDLIB_VERSION
   sha `092dfa0` + notes JSON (6) + gate record); PR CI RUN `38786292404`
   in progress. ON GREEN: merge xiom#6 -> tag v0.64.5 on the merged main
   -> release run -> verify assets + SHA256SUMS -> record + goal_report.
   UPDATE 2026-10-15T18:45Z: PR CI round 2 (`38786292404`) -- windows
   GREEN; two leftovers FIXED on main: m332 `6aa7dc4f` (ubuntu
   `cxx_standard_applies_to_c_source_cpp`: C++ --c-source link now adds
   `-lstdc++`/`-lc++` on non-Windows; + rustls vet exemption 0.23.46 for
   cargo-vet). Pin branch REBASED (`0ff49434`) + force-pushed; PR CI
   ROUND 3 = run `38787764451` (queued). Same next-steps on green.
   UPDATE 2026-10-15T19:00Z: round 3 -- windows + hygiene GREEN (vet +
   advisories fixed); ubuntu red on FIVE g2 fault locks (first ubuntu
   exposure: llvm.trap signal-terminates on unix -> `code()` None; the
   m244/m308 "[trampoline] fault" marker is Windows-VEH-only). FIXED m333
   `81e98099` (platform-aware trap locks: unix None accepted, Windows
   strict; marker asserts cfg(windows), containment exit-0 asserted
   everywhere). Pin branch rebased (`247b0f14`) + force-pushed; PR CI
   ROUND 4 = run `38790060894`. v0.64.6 QUEUE ADDED: POSIX fault-trace
   marker + cross-OS fault-containment observables (stdlib runtime).
   ON GREEN (round 4): merge xiom#6 -> tag v0.64.5 -> release run ->
   verify assets -> record -> goal_report complete.
   UPDATE 2026-10-15T19:13Z: PR CI ROUND 4 `38790060894` ALL GREEN
   (ubuntu + windows + hygiene). PR #6 MERGED via rebase-merge (repo
   disallows merge commits) -- origin main = `84f761c5` (the pin commit).
   TAG `v0.64.5` (annotated) PUSHED on `84f761c5`; RELEASE RUN
   `38792051847` in progress.
   *** RELEASED 2026-10-15T19:20Z ***: release run FULLY GREEN (guard +
   linux-x64 + windows-x64 + macos-x64 + macos-arm64 + VSIX + publish).
   GitHub Release `v0.64.5` live + non-draft: SHA256SUMS, the four
   platform archives, VSIX 0.12.2, wasm bundle + typings. Pin:
   STDLIB_VERSION = stdlib-v0.64.5 sha `092dfa0` (tag obj 1667b3a8) +
   notes JSON (6 highlights) + gate record; pin-gate fix round m328-m333
   shipped in the tag. stdlib side: xiom-std 0.64.5 published. v0.64.6
   headline queue: general Darwin work (declare landed m331; runtime tf
   x64 / _SC_AVPHYS_PAGES / arm64 fp128 asm + engine macOS build/run
   evidence) + POSIX fault-trace parity (m244/m308 markers). Post-release
   wave: archive reporter re-verification (bindings/orbitdb/playground/
   engine/stdlib rows) per the candidate record.
  Row-sweep note: several check-file rows still show `acked`/red for items
  fixed in m276-m284 (e.g. 1437/1512 -> m279; 2230/2053/2137/1402/1403/
  1418 -> m276-m282) -- sweep rows when convenient.
- OPS LESSONS (all cost time this session; obey):
  - LINE ENDINGS: `core.autocrlf=true` and repo docs stored CRLF (e.g.
    COMPILER_BUGS.md). PowerShell `Set-Content`/`-replace`/append silently
    converts whole files (whole-file diff). Edit markdown with the `edit`
    tool or Python `io.open(newline='')`; verify `git show --numstat` is
    small before committing; fix with `git update-index --cacheinfo` of
    the raw blob if it happens.
  - STDLIB RESOLUTION: multiple trees on this box (pinned
    `E:\xiom-lang\xiom\stdlib`; live `E:\xiom-lang\stdlib`). For
    `target/debug/xiom.exe`, exe-ancestor order picks the LIVE tree, and
    29 scanned duplicates can mix bodies across trees. ALWAYS set
    `XIOM_STDLIB=E:\xiom-lang\xiom\stdlib` for manual probes. Prefer
    `--manifest-path` + `cargo test` harness for locks.
  - WSL CROSS-PLATFORM CHECKS (works): run the Windows compiler FROM WSL
    via interop with WINDOWS-style paths (inputs `E:\...`, not `/mnt/e/...`)
    and `XIOM_STDLIB=E:\\xiom-lang\\xiom\\stdlib`; it links with the Linux
    clang and produces Linux binaries (`E:\xiom-lang\xiom\target\tmp-*` as
    the scratch dir). WSL user: `lefteris`; repo at `/mnt/e/...`; released
    toolchain in `~/.local/share/xiom/bin/`.
  - BINARY CONFUSION: `xiom-dbg` (in the toolchain bin/) is the DAP debug
    adapter -- ignores `--check`, exits 0 always. Never gate on it; assert
    the compiler sha256 in harnesses.
  - HOST: `dbg125p.exe` runaway (94 GB) kills cargo/CLR; `taskkill /F /IM
    dbg125p.exe`. `xiom-check --lib` needs `--test-threads=1` under memory
    pressure (serial 199/199).
- Relay bus SHAs: ... `438b714` (m281), `40e3c40` (m282), `9b4ec0d`
  (allocator triage), `fbda2a7` + `7aa7bfa` (m283/m284 closers + B-21),
  `da3160b` (stdlib-6 cross-platform verify), `61dfda8` (orbitdb-1358
  diagnosis), `5a2f5d2` (m285 fix + row), `deb173d` (m286 fix + row),
  `b8caee2` (m287 fix + row), `ce5f2b4` (bindings-4 attempt triage),
  `76869eb` (stdlib-4 fix + lock row), `bb30a68` (stdlib-5 fix + row),
  `d5e84f8` (stdlib-6 part-a fix + leg-b triage), `170cdd9` (leg-b
  attempt-2 triage), `9712942` (m291 enforcement + stdlib-7/4 correction).
  NEXT-BATCH FIX REFS rows + history bullets current.
- Selfhost: phase-4 worktree (ses_eded8c4d8ffe, sole writer) idle after its
  13:00Z prompt; sibling ses_ef8e3a27cffe idle. Rebases/tests will pick up
  m276-m284 at the next rebase.
- Verification battery (current): e2e g1/g2 filters at ci.yml lines
  152/153 (g1 = 16, g2 now 174 incl. m283/m284 locks); front-locks = the
  ci.yml CLI list (now incl. `ffi_ptr_arith`, `ffi_ptr_slot_readback`,
  `null_literal_marshal`, `ffi_fn_env_cast`); `xiom-check --lib` (SERIAL),
  `xiom-codegen --lib`, `xiom-graph --lib` (36), `xiom-pkg` (79). Capture
  compiler output to files. Last full green: e2e 16+174, front-locks,
  libs 199/36/79.

Landed in the m254-m275 release wave (historical, already pushed):

- `677d65a7` m261/m262 module-local call precedence + module-private const
  identity. Closes REL-20261011-1124-orbitdb, REL-20261011-1052-stdlib.
- `fde5548e` m263-m265 fn-value ABI: C-boundary marshaling (B-13/1904),
  Vec[fn] element-call arity + unit returns (B-14/B-15), module-scope
  fn-ptr storage (0317-pulse-7). CLI lock `crates/xiom/tests/ffi_fnptr_abi.rs`
  (real C host) + e2e m264/m265.
- `0b1c6374` m266/m267 unresolved imports are hard single errors (2124 +
  1802 gate) + bare pattern variants prefer the scrutinee's enum (twin).
  Locks: `checker_locks::m266_unresolved_import_one_clear_error`, e2e m267.
- `c86e2f2d` m268/m269 install-target validation + deterministic tar (2100),
  packages-bearing home preference (2102).
- `0219c5e3` m270 pulse site-mode AV root-caused to the m261-class bare-call
  hijack of a module-local struct-returning helper; lock e2e m270 + fixture;
  the 1854 COMPILER_BUGS section flipped to FIXED. Closes 1854 + 2122-pulse.
- `f69b542e` m271 float Vec slot stores narrow numeric sources (XVC-C-15):
  closes REL-20261011-1134-xvector. Lock e2e m271 + fixture; IR witness
  (fptrunc/sitofp); relay item filed `fixed`, release-check row appended.
- `b1ee2f4f` chore(release): v0.64.4 prep -- workspace + SELFHOST_VERSION
  bump, `release-notes/v0.64.4.md` draft (4 compiler highlights; dry-run
  convert 6 total), VSIX README pin.
- `745ff47e` m272 fn-value ABI regressions found by the gate's full e2e:
  C-boundary-only `callee_is_extern` unwrap gate + module-qualified
  `fn_typed_params` keys; locks = the six tests added to ci.yml.
- `c44d9fae` m273: m219 IR-shape lock follows the m263 `i64 ()**` spelling.
- `b873c6d6` m274: B-18 fix (bare fn NAME to an extern fn-typed param
  passes the raw code address; the four call-site wrap paths are gated by
  `callee_is_extern`); m263 CLI lock gained the bare-name arm. B-19
  resolved `wontfix` -- package-side `UInt8` vs `Uint32` declaration for
  SDL_WasInit (see COMPILER_BUGS m274; probe/package evidence).
- `4c42fc0a` m275: stdlib wave-regression fix (REL-20261011-2045-stdlib):
  the As-cast env unwrap now requires the compiled value to be the i64 env
  slot, so a fn-typed param name leaked into closure_locals can no longer
  hijack a float's cast (`Float32.to_float(v) { return v as Float64; }`);
  collision lock added to the m263 CLI fixture.
- `df493530` chore(release): v0.64.4 pin + notes JSON + gate record (the
  release commit); `91860f11` released-record docs. Gate record:
  `docs/RELEASE_GATE_v0.64.4.md`.

## 1b. Release wave state (next actions)

- PIN DONE (2026-10-11T22:38Z): stdlib tagged `stdlib-v0.64.4` = `3df0843`
  and verified (witness green on the v0.64.4 archive from the `87c6ca3c`
  push; metro revert `4e8b696`; refresh run 38177738188: corpus 954/954,
  probes 288/288, modules 509/509, barename 0/509;
  REL-20261011-1812-compiler verified + closed).
  - `STDLIB_VERSION` = `3df0843b2c384f9768867d96cfd4a611bed3034b`; vendored
    `E:\xiom-lang\xiom\stdlib` is a worktree pinned at the tag;
  - `release-notes/v0.64.4.json` converted + verified from the clean tag
    checkout (6 highlights = 4 compiler + 2 stdlib, 0 breaking, 2 docs);
  - origin/main = `df493530` (the release commit; do NOT push again until
    the tag run settles).
- RELEASED (2026-10-12T13:37Z): tag `v0.64.4` on `df493530`; release run
  `38256358845` GREEN; assets live (linux-x64, macos-x64/arm64,
  windows-x64, VSIX 0.12.2, wasm bundle, SHA256SUMS).
- WAVE LANDED (2026-10-12 evening): `release-checks/compiler-compiler-v0.64.4.md`
  (created 15:01Z) carries the reporter verdicts -- bindings, packages,
  orbitdb, playground, stdlib filed; successes and the ~35 still-red rows
  are distilled in section 1c. The to-all item `REL-20261011-1731-compiler`
  stays open until every row has a verdict (no compiler action needed for
  that bookkeeping).

## 1c. Next program (v0.64.5) -- plan from the v0.64.4 release-check

The v0.64.4 verdict wave is landing (`release-checks/compiler-compiler-v0.64.4.md`,
created 15:01Z). Wave closed: B-13/1904/B-14/B-15/B-18/B-19; m260 casts
PART B; C-ORBIT-06/07; 2100-pkg WSL paths; 1548-stdlib-2/5; packages -4/-6;
1900-engine + 1803-selfhost (fixed in the wave); 2045 (m275). Still red /
open: ~35 rows. Plan:

TRIAGE FIRST (P0):
- T1 `REL-20261012-1418-pulse`: DONE (m276, this commit). Not a v0.64.4
  wave regression but the m263 env-unwrap class on the EMPTY-store path:
  kv_open -> _create_segment -> stdlib write_u32_le's `var x = v as Int`
  unwrapped a leaked closure name (`v`); m275's i64 gate missed the
  zext'ed UInt32. Fix = type veto in `unwrap_fn_env_for_c`. Witness:
  tests/regression/m276_env_name_leak_cast (pre-fix 0xC0000005, post-fix
  exit 0) + PULSE empty-store serves + creates seg-1. (The reporter's
  "exit 0" was a %ERRORLEVEL% parse-time artifact; it AVs at 0xC0000005.)
- T2 `REL-20261012-1630-bindings` B-22: DONE (m277, this commit). Root
  causes: (a) nested fn-type markers split at `find(") -> ")` +
  `split(',')` (the `unknown type 'Int32, Int32) -> Int32'` family,
  1403-packages) -- now depth-aware `split_fn_marker_body`/
  `split_top_level_type_params`; (b) cast-call fn-typed args kept the
  closure env, and the env field 0 is the env-first THUNK (handing it to
  C shifts every arg) -- `wrap_fn_ref_env` envs are now two words
  (0 = thunk, 1 = RAW code) and every C boundary (`f as Int`, extern
  fn-typed params, cast-call args) loads field 1. Witness:
  tests/regression/m263_fn_value_c_abi gained via_cast_param /
  via_cast2_param; probe9 cb-abi green (42/42/42); pre-fix exit 9 + the
  warning, post-fix exit 0, no warnings. Closure-to-C stays on the
  workaround APIs (closures were never C-callable).
- T3 `REL-20261011-2137-engine`: DONE (m278, this commit). Not symbol
  mangling but TYPE identity: the catalog injection deduped same-named
  types by BARE name, so `xiom.<root>.render.mesh.Mesh` vs stdlib
  `xiom.geom.geometry_3d.Mesh` were decided by ASCII module order
  (`Runtime` < `geom` < `runtime`/`rt` < `zz`) -- the loser compiled
  against the winner's layout (field 2 vs 0) -> AV. Fix: collision
  pre-scan + module-qualified injection of colliding types,
  `pick_deterministic` proximity (current module, then entry `use`
  imports via the new `primary_imports`), and injected flat fns derive
  module context from their leaf-qualified names. Witness: engine
  scratch `engine` and `zz` roots both exit 0 (zz was 0xC0000005);
  lock `e2e_m278_type_name_collision` + fixture
  `tests/regression/m278_type_name_collision/`.

THEMED BATCHES (m-numbering assigned as they land; one commit per batch):
- FFI pointer: B-16/B-17 DONE (m279) and 2230 DONE (m280: unsafe Copy-Out
  Str-copied raw-pointer returns -> gated on !expr_is_pointer at all three
  sites; temporary Vec[Str] element concat -> vec_value_xiom_type Call arm;
  lock e2e_m280_unsafe_ptr_temp_str). Remaining in this batch:
  B-11/0304/1548-bindings-2 (out-param recycling / foreign buffer -- not yet
  triaged) and B-23/B-24 (REL-20261012-2051-bindings-2: null literal to C;
  ffi out-slot reads after a C write).
- Thread/spawn runtime: 1627/1402-xvector + 2053-engine (fn-value runtime
  entrypoint AV; marshal half fixed at v0.64.4).
- Allocator/runtime: 2032-engine (Vec drop leak), 2320#1 (with_len HBO,
  ASAN), 1548-bindings-5 (confined alloc/free spin).
- Borrow/codegen: B-21 (undefined IR temp on &mut of struct-returning
  call), 1659-orbitdb (copy elem-assign aliasing), 1549-packages-9 (bare
  &mut Int reads pointer).
- Resolver/checker: 1548-bindings-4, 1457-engine (bare sqrt), 2320#3
  (4-segment const), 1548-stdlib-6, 1550-orbitdb-2 + 1358-orbitdb
  (--check T001 + Linux rc), 1709-xvector, XVC-C-13, 1053/1456 (hash /
  Mutex shadow), 1626.
- Contracts/clauses: 1549-packages-8 (recursion AV), 1549-packages-3
  (unsafe return cast null), 1549-packages-2, 1411-engine, 1548-stdlib-4/7,
  1549-stdlib-2..7 (payload reads, polyhedra, cross-type callbacks,
  shape-mismatch silent).
- Tooling/registry: registry B1/B2/trust/resolve, 1549-packages-5/7,
  1500-benchmark cache ask, website-7/8/9, benchmark trampoline row, pulse
  RSS, playground rows (coercion/buffering/drvfs).
- macOS leg: 1549-pulse/1609-pulse typed `@llvm.memset.p0i8.i64` emission.

## 2. Batch 6 (LAST) -- FLOAT32 SLOT STORES -- DONE (m271)

- Fix (`f69b542e`): `stmt.rs` Vec indexed/field store converts numeric
  sources to the element float width before the bit-level elem store
  (float sources via coerce_value -- fptrunc/fpext; Int via val_to_i64 +
  sitofp); the same branch widens Int literal stores into Vec[Float64].
- Witnesses: pre-fix exit 3 (A `a[1]=0`, B `b[1]=9.80908925027372e-45`,
  E `e.data[1]=0`; C/F/G green) -> post-fix exit 0; IR `fptrunc double 7.0`
  at the A site, `sitofp i64 7` at the B site. The xvector probe needs the
  lane-local `Vector` type; the local-struct matrix reproduces the same
  values (scratch recipe: `%TEMP%\kilo\m271\repro.xi`).
- Lock: `e2e_m271_f32_slot_store` + `tests/regression/m271_f32_slot_store/`
  (fails Some(4) pre-fix), ci.yml group 2.
- Relay: `REL-20261011-1134-xvector` acked -> fixed with evidence; to-all
  status item updated; release-check row `m271 | f69b542e` appended
  (relay commits `a3e9508` + `6386301`). Remaining gate: reporters re-run
  their probes at the archive and set `verified` or bounce.

## 3. Selfhost track (parallel)

- Worktree: `.kilo/worktrees/selfhost-phase-4-codegen` (branch
  `selfhost-phase-4-codegen`); session `ses_eded8c4d8ffeMNzBsYzBYIgAO8`;
  sibling session `ses_ef8e3a27cffeKY1Sz63tGGas2t` must stay IDLE.
- State (last confirmed 2026-10-11 ~19:40Z; re-check with agent_manager
  before relying on it): phase 6 at **T6 4/64** (P0-P1e + P5a landed; all
  five gates re-run green; the relay item `REL-20261011-1525-selfhost`
  records P1a/P1b/P1c/P5a + T3 meter 3/64). Remaining: P1 remainder
  (method receivers: self alloca + field GEPs + `x.method()` calls) ->
  P2 tuples / P5 stack arrays -> phase-6 meter flip (4 of 11).
- Cadence: the session works a chunk then idles. Check with agent_manager
  `list`; if idle, prompt: continue the next unchecked stage per
  `docs/checklists/selfhost-phase6.md`, rebase onto the newest LOCAL main
  before full parity runs, sole writer, same commit/doc/relay discipline.

## 4. Release coordination -- DONE (historical)

v0.64.4 coordination complete: `REL-20261011-1731-compiler` (to all;
status + updates 1-6), `1731-compiler-2` (to stdlib), `1812` (pin
decision; verified + closed), `2014` (push nudge; fixed). The only
follow-up is bookkeeping: close `1731` once every release-check row has a
verdict (the check file is the source of truth). Nothing else pending.

## 5. Verification command set (copy/paste)

- `cargo build -p xiom`
- e2e groups from ci.yml (PowerShell; the `-split` matters):
  `$l1=(Get-Content ".github\workflows\ci.yml")[152]; $fa1=($l1.Substring($l1.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa1`
  `$l2=(Get-Content ".github\workflows\ci.yml")[153]; $fa2=($l2.Substring($l2.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa2`
  (currently 16 + 167 green; the m272 six joined group 2)
- front-end locks: `cargo test -p xiom --test doctor_cli --test borrow_e001 --test checker_locks --test run_script_cli --test cxx_standard_cli --test ffi_fnptr_abi`
- units: `cargo test -p xiom-check --lib` (199, incl. full catalog corpus
  gate), `-p xiom-codegen --lib` (12), `-p xiom-graph --lib` (36),
  `cargo test -p xiom-pkg` (79).
- A/B baseline switch: `git checkout <sha> -- crates/` then build; restore
  with `git checkout HEAD -- crates/`. Attribution trick that worked
  repeatedly: disable ONE fix's effect (early `return None`) + rebuild +
  retest.

## 6. Ops notes for the fresh session (learned this session)

- NEVER pipe compiler output into `Select-Object -First N` directly -- the
  broken pipe kills the compiler (exit -1). Capture with `*> file` then
  grep the file.
- `%TEMP%\kilo` holds many stale toolchain checkouts -> W001 flood noise
  (harmless; the chosen stdlib is the pin). Suppress with `2>$null` when
  only the exit code matters.
- A/B runs: use the PINNED stdlib `E:\xiom-lang\xiom\stdlib` (unset
  XIOM_STDLIB); `E:\xiom-lang\stdlib` is the stdlib lane's moving working
  repo (commits while you work).
- Pulse 1854 repro: `git -C E:\xiom-projects\xiom-pulse archive 01dcd1b`
  into a fresh temp dir + append the unused `out_text` helper to
  `tests/test_app.xi`. A stale `a.exe` in the tree breaks 8 resource tests
  on ANY compiler (delete it, or compile with `-o` to a path outside the
  tree); fresh extracts are clean.
- bash tool: `workdir` must EXIST before the call (create the dir from its
  parent first).
- `m211_listdir_probe/` at repo root is another lane's untracked scratch --
  leave it. `.kilo/worktrees/*` belong to their sessions -- do not touch.
- relay.py: `new --body-file` for item bodies; `fix --note` inline; `commit
  --lane compiler [-m] <ids> --path <repo-file>` when a non-item file (e.g.
  a release-check) changed too.
- Diagnostics for unresolved imports: env `XIOM_TRACE_IMPORT` prints
  `[imp]` traces (kept in-tree, env-gated).
- `gh` CLI IS available: `gh run list -L 8`, `gh run view <id>`, `gh run
  watch <id> --exit-status`, `gh release view v0.64.4 --json assets`.
- The vendored stdlib is now a proper git WORKTREE pinned at `3df0843`
  (not the old stale checkout). To re-pin later: `git -C
  E:\xiom-lang\stdlib worktree prune`; rename the old dir aside; `git -C
  E:\xiom-lang\stdlib worktree add --detach E:\xiom-lang\xiom\stdlib
  <sha>`; delete the old dir after verifying.
- Push regime: NEVER push without the owner's authorization. The v0.64.4
  pushes (20:21Z, 21:10Z, tag, release-record) were owner-authorized; the
  doorbell cron may also push staged work -- ALWAYS `git fetch` and check
  `origin/main` first.
- The scheduled "XIOM Heavy Suites" workflow is PRE-EXISTING broken
  (rustup/cargo-fuzz setup; ~45s failure on every run since 2026-09-21) --
  ignore it for release purposes.
- Lesson from m275/2045: `local.closure_locals` (and the other
  name-keyed maps) are NOT function-scoped -- when adding a new consumer,
  gate on the compiled LLVM type where possible (an env is always the
  `i64` slot) instead of trusting the name set.

NEW (2026-10-13, v0.64.5 triage + FFI batches):

- Reporter "exit 0" from `cmd /c "... & echo EXIT=%ERRORLEVEL%"` is a
  PARSE-TIME artifact (expands before the app runs); capture exit codes
  with `cmd /v /c "... & echo EXITCODE=!ERRORLEVEL!"` or PowerShell
  `$LASTEXITCODE`. T1's "silent exit 0" was really 0xC0000005; WER can
  hold the process so a short watchdog misses it -- check the Application
  Error event log.
- Control the PRECONDITION state in repros (T1: the kv AV only fired on
  an EMPTY store dir; once one run created seg-1 every later build
  served, silently invalidating a whole bisect round).
- The loader has TWO module paths: EAGER nested Module decls (small
  single-file/dir fixtures -> qualified type keys) vs the LAZY catalog
  injection (`collect_external_decls` -> flat, bare-deduped). A
  single-file fixture often CANNOT reproduce injection-path bugs; use a
  real project tree A/B (the engine scratch copies) as the witness.
- xiom-check's R39/R44 `colliding` triage (shape rules; identical-layout
  facade duplicates like the two stdlib AtomicInt decls stay bare) is THE
  collision authority -- derive new injection-side logic from that map; a
  hand-rolled bare-name pre-scan broke t2-queue (AtomicInt).
- Builtins (as_ptr/as_mut_ptr, null, ...) need an INFERENCE counterpart:
  call.rs lowers them, but bindings stay untyped unless
  infer_call_return_xiom mirrors them -- and `i8*` is ambiguous (Str vs
  *UInt8), so gate operator lowering on AST-level pointer-ness
  (`expr_is_pointer`), never on the LLVM type alone (Copy-Out Str-copied
  a *UInt8 return; `base + N` became str_concat).
- relay.py notes: PowerShell passes argv to python; EMBEDDED DOUBLE
  QUOTES break the argument (the note split at `->`). Use quote-free
  notes.
- cargo from the wrong cwd fails: use
  `cargo build --manifest-path E:\xiom-lang\xiom\Cargo.toml -p xiom`
  when the probe shell sits in another repo.
- Fixture conventions: pure-XIOM checks -> e2e fixture + ci e2e group 2
  filter; anything needing `--c-source` -> CLI test under
  crates/xiom/tests/ (ffi_fnptr_abi.rs pattern) + the front-locks list.

## 7. m-numbering

- m261-m292 + m293-m299 + m301-m309 + m310 LANDED; m311/m312/m313 are
  reserved by the in-flight fixers (O2 contract barrier, spawn readback,
  ffi.free guard lowering); **next free is m314**.

## 8. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), continuing the v0.64.5
program after the candidate-closing batches m276-m284. Read
docs/SESSION-HANDOFF.md FIRST (CURRENT STATE block at the top, the ops
lessons, and the remaining-queue list); consult docs/RUSTC-STUDY.md +
docs/rustc-study/ for reference patterns; rustc source is E:\repos\rust
(READ-ONLY; no git writes there).

STATE: v0.64.4 released (df493530) and verified. Git: origin/main =
91860f11 (fetch first -- the doorbell may push staged work). Local main = the latest commit (see git log), AHEAD by ~68 unpushed
commits (m276-m292 + m293..m299 + m301..m309 + m310 + docs + attempt logs); do NOT push
without the owner's authorization. Relay bus: latest compiler commits
97e524e/3554774/ec470bb (+ the m292..m310 fix notes); the check
file release-checks/compiler-compiler-v0.64.4.md is the candidate record
(the 17-item engine batch was queued into it on 2026-10-14).

CLOSED this program (reporter re-verify at the next archive): T1-T3,
B-17+B-16, B-23+B-24, B-11+B-20 readback family, 1549-packages-3,
1549-stdlib-5, 1549-stdlib-7 (cross-type generic callbacks), 1548-stdlib-3
(tostring import no longer hijacks adapter closure fields), 1548-bindings-4
(m292 worklist guard), 1548-stdlib-4
(@pre on &mut param fields -- real fix m291), 1548-stdlib-7 (guard-form
ensures + enforcement restored), 1548-stdlib-5 (annotated Slice locals),
B-25, B-26,
B-21, 1549-packages-8 (contract cycles), 1549-stdlib-6 (cross-platform
verified), 1358-orbitdb (not a compiler bug -- xiom-dbg binary confusion
diagnosed).

NEXT -- first INTEGRATE the pending fixer results, then start the engine wave
(witness-first triage; see the check file + the bus for details):
0. PENDING INTEGRATION (Agent Manager fixer worktrees; check `agent_manager
   list` + `git log --oneline main..fix/<branch>`):
   a. m313 fix/m313-ffi-free-guard (ses_ec537c634ffej3MZekLVoymbeR): COMMITTED
      32626879 -- ffi.free call sites lower through the arena-aware wrapper
      (the compiler half of REL-20261013-1445). Integrate first: cherry-pick -n,
      ci.yml token(s), COMPILER_BUGS section, build, battery, ONE batch commit;
      then ACCEPTANCE: compile %TEMP%\kilo\threadcheck\spincheck.xi with
      XIOM_STDLIB='E:\xiom-lang\stdlib' --force --no-cache, run with a 15s
      watchdog; must print v=7 fast. If green, append a compiler note to
      REL-20261013-1445-compiler (compiler half landed + acceptance green;
      alloc/free workarounds retirable at the next pin) + check-file bullet +
      relay.py commit --path.
   b. m312 fix/m312-thread-spawn-readback (ses_ec5377b3effeDu3aCdH56UcotP):
      no commits yet at 14:55Z. On idle+commit: integrate; then re-run
      dbg131s.xi/dbgworker.xi vs stdlib main (expect `joined 42` stable x5;
      probes in %TEMP%\kilo\threadcheck\; rawspawn/rawgeneric/structret are
      the isolation set). If green: phase-2 note on REL-20261013-1423-compiler
      (compiler half green; thread gate 2053/1627/1402 re-runnable at the next
      pin) + bullet + relay commit.
   c. m311 fix/m311-o2-no-fold-barrier (ses_ec54222e7ffeO1IOB0h3cLS72X): no
      commits yet. The -O2 contract constant-fold pruning finding
      (COMPILER_BUGS ~15886); no bus item -- fold into COMPILER_BUGS +
      handoff + a check-file history bullet.
1. ENGINE RELAY BATCH (17 items queued into the check file 2026-10-14,
   untriaged -- the next fixer-wave candidates, natural clusters):
   2042 + 2042-2 + 2227 (UInt64 bit-63 + Int128/UInt128; 2042 is HIGH),
   1158 (ident named spawn drops fn -- m292 catalog leaf-shadow class),
   2139 (enum-variant/geom name collision -- m298 class), 2256 + 2324-2 +
   2324-3 (constant surface: shadow hang, var fn export drop, diamond const),
   2215 + 2215-2 (Vec push in match arm; move-semantics pin), 2238 x3
   (CLI lane: vecvecstr / fnvalue_records / meshencoder), 1819 (Int ptr arg
   B-16 class), 2035 (&Str param spelling), 2119 + 1213 (observational/low).
   Keep 3-4 fixers in flight.
2. Everything else (m276-m310) is DONE and relayed; remaining non-code
   gates: owner push authorization (~68 unpushed commits), a pin carrying
   stdlib main + this tree, the archive reporter re-verification wave
   (incl. thread gate + spin acceptance), 2032 owner decision, macOS pair
   (needs a new archive), 1549-packages-2 faithful repro, release-gate record.
   The stdlib-lane verification notes for 1423/1445 are ALREADY on the bus
   (97e524e: 1423 hang gone/readback localized; 1445 spin + call-site
   finding) -- only the phase-2 green notes remain.

Number from m314 (m311/m312/m313 reserved); ONE commit per batch (fix +
fixtures + docs + COMPILER_BUGS section) + a relay fix-note with numeric/IR
witnesses + a check-file row.

RULES: reproduce from the item's witness before touching code; battery =
e2e g1/g2 filters at ci.yml lines 152/153 + the front-locks CLI list
(incl. ffi_ptr_arith/ffi_ptr_slot_readback/null_literal_marshal/
ffi_fn_env_cast) + xiom-check --lib -- --test-threads=1 + xiom-codegen +
xiom-graph + xiom-pkg; A/B with the PINNED stdlib by setting
XIOM_STDLIB=E:\xiom-lang\xiom\stdlib explicitly (exe-ancestor order
otherwise picks the live tree and can mix module bodies across trees);
capture compiler output to files; relay notes must be quote-free
(PS->python argv); use cargo --manifest-path when the shell sits outside
the repo; fixture convention = pure-XIOM -> e2e fixture + ci group 2,
--c-source shapes -> crates/xiom/tests CLI test; edit markdown with the
edit tool or Python (PowerShell text appends convert line endings and
produce whole-file diffs -- verify git show --numstat before committing);
WSL checks run the Windows compiler with Windows-style paths and link
Linux binaries; do not gate on xiom-dbg (DAP adapter, exits 0); house
voice; ASCII; never push without owner authorization.

HYBRID RECON (2026-10-13, owner-directed): fan out READ-ONLY triage to
subagents to compress the queue. Pattern that works: 3-4 parallel `general`
subagents (Task tool), one per item -- each reads its relay item + witness,
reproduces with the PINNED stdlib (`XIOM_STDLIB=E:\xiom-lang\xiom\stdlib`)
using the already-built `target/debug/xiom.exe`, and returns a concise
report: repro rc, root cause with file:line, minimal fix proposal, blast
radius, and a suggested lock. HARD LIMITS for subagents: NO file edits, NO
cargo builds (the main lane owns the target dir), scratch only under
%TEMP%\kilo\; the main session applies and verifies every fix serially (ONE
commit per batch, full battery). Integrate the topmost tractable diagnosis
first; the two decision items (1548-stdlib-6 leg b, 1548-bindings-4) each
have a precise attempt log in docs/failed_attempts.md.
FIX WAVE (2026-10-13, owner-directed): fixers now run in isolated Agent
Manager WORKTREES (m293-m295 branch seeds: packages-11, packages-9,
stdlib-6 leg b) -- each fixes + tests in its own worktree and commits
source+tests only; the main lane cherry-picks serially and owns
docs/relay/CI rows + the final full battery.

PARALLEL SELFHOST: phase-4 worktree (ses_eded8c4d8ffe) is idle and the
sole writer there; sibling ses_ef8e3a27cffe stays idle. Prompt it only if
it needs to continue docs/checklists/selfhost-phase6.md on the current
main (post-m284).
```
