# Next candidate plan -- v0.65.2 (2026-10-27)

STATUS: planning + in progress. **v0.65.1 is production** (tag
`c4329915f6a55e994ad7ddc05399059a85a0c796` on `f2c01cf9c`; release
40388025015; 11 assets; registry `xiom-std 0.65.1`). No v0.65.2 tag until
the gates below are green and the owner gives GO. v0.64.9 stays withdrawn.

## Theme

Two fleet blockers plus the retention floor: unblock arm64 builds, remove the
Vec 8x slot waste (t1), and start the non-Vec ownership work (Str/Map/JSON/
HTTP). Selfhost Phase 6 continues in parallel; LSI hardening is scoped with
the benchmark lane.

## P0 -- arm64 Result[Str, E] contract payload

**DONE and merged to main** (`3e775908a`, merge `1d4de2a1a`). Compiler-side
gate is green.

- Cause: `expr.rs::emit_payload_rebind` loaded the Result payload slot as
  `i64` unconditionally; arm64 stores `Result[Str, E]` field 1 as `i8*`.
  `emit_is_payload_literal_cmp` had the same hardcoded load.
- Fix: `load_payload_as_i64` loads the field's actual LLVM type and
  normalizes to the erased handle (`ptrtoint`/`bitcast`/`zext`/`trunc`).
- Lock: `e2e_m3752_arm_result_str_contract_payload` runs in the arm64 floor
  job (ubuntu-22.04-arm, clang 14) with `e2e_m1101` (variadic printf).
- Evidence: full e2e **2648/2648 (4 ignored)** on debug and release; CI
  **40411862697 SUCCESS**, including the **arm64 floor**.
- Exit: PULSE `linux-arm64` + `macos-14` build/link/run green on the v0.65.2
  archive; `REL-20261024-1101-pulse` verified. (The compiler gate is green;
  the fleet app-build gate remains.)

## P1 -- t1 memory: Vec packed element slots

The t1 Peak Memory row is XIOM 30 MB vs C 8 / Go 3 / Rust 3 / Zig 2 for the
4096/32 buddy fixture; the pool is not the cause.

- Cause: `Vec.with_len` zero-fills `n*8`; `push`/`reserve`/`insert` grow by
  `new_cap*8`; `shrink_to_fit` uses `len*8`; the compiler floors slots at 8
  (`vec_slot_size = max(elem_size, 8)`).
- Ownership: stdlib item `REL-20261026-2142-compiler`; compiler
  `vec_slot_size` change follows the stdlib tag (never one side alone -- the
  current n*8 zero-fill overflows a packed allocation).
- **Combined status (2026-10-27):** stdlib fixed the branch at `863248f`
  (`with_len` reads `v.elem_size`; the type receiver was the segfault). With
  compiler `c60729714`: `p_wave289_vec8` exit 0; 10/10 memory gates; the
  4 MiB `with_len` under the 24 MB cap; full e2e **2648/2648 (4 ignored)
  on both**. The stdlib tag is the **owner gate**; the compiler half stays
  on `fix/v0.65.2-vec-packed-slots` until it lands.
- Gate: `Vec[UInt8].with_len(4M)` RSS < 5 MB; t1 Peak Memory < 5 MB on the
  4096 fixture; ASAN/guard-page overflow check for `with_len`; full e2e.

## P1 -- retention beyond Vec

Order: Str -> Map -> JSON -> HTTP. Vec is flat; the floor is the non-Vec
ownership path. Related items:

- `REL-20261010-1549-pulse-4` (Linux request-path RSS growth; v0.65.1
  remeasure: /health 50 KB/req, /serialize 36, /insert 78,
  /serialize/big 219).
- `REL-20261020-1807-orbitdb-2` (write-path/COW, 895 MB/200k),
  `REL-20261020-2102-orbitdb` (btree 18 MB, WAL 203 MB, soak 895 MB),
  `REL-20261021-0423-orbitdb` (607 MB/151s at 200k),
  `REL-20261021-0606-orbitdb` + `REL-20261021-1100-orbitdb` (1M: cache
  thrash, then 2,494.7 MB pass; remaining 880 s teardown).
- `REL-20261010-1609-stdlib` (inline Result.value[i] payload index).
- `REL-20261026-2247-stdlib` gate-9: only **1609** red (it also blocks
  `1549-pulse-5`); the full stdlib v0.65.1 battery is green. The **1704**
  JSON matrix is linear ~2.3 KB/iter at 10k-50k: parse 10k 23.6 MB -> 50k
  113.7 MB, obj 23.7 -> 107.6, type 23.4 -> 85.2, get 23.1 -> 97.9; 200k is
  cap-killed. The 10k/256 MB target is met; the JsonValue enum-tree drop is
  compiler-side and is the JSON/Str retention target.

Gate: Str and Map per-op flat; JSON parse loop within an agreed cap; HTTP
`/health` and `/serialize/big` sustained-request caps; A6 1M peak at or below
1.8 GB plus a measured teardown.

## P1 -- selfhost Phase 6 (parallel)

Worktree `E:\xiom-lang\xiom\.kilo\worktrees\selfhost-phase-4-codegen`
(branch `selfhost-phase-4-codegen`; T3 83/83, phase-6 14/64 at the RD11
baseline).

- **DONE 2026-10-27:** value-returning `NkExprMatch` + parser tail re-wrap
  (`67b0635a5`) -> `diff_ast` **83/83**, T6 14/64; `Vec.len()`/`&Vec` index
  reads (`743a28d8e`) -> T6 **16/64**. T3 green.
- Next: `Vec.pop` (`m37_round7_vec_pop_slot`), `&Vec`+Float64 element typing,
  Option/payload matches (m1136 `infer_pattern_binding_type`).
- Gate: T3 83/83, T6 count non-decreasing, T1 + `diff_ir_headers` green,
  ASCII guard, one commit per stage with checklist/progress evidence.
- No release dependency; never push from the worktree.

## P1 -- LSI / safety hardening (scoped)

Current t8/t9 schema-2.0 profile: 12 probes, LSI 45; UAF and
uninitialized-read are SILENT_UB; type confusion is SILENT_FAILURE with the
sentinel changed; everything else panics. Compile prevention 0%, runtime
detection 75%, survival 25%.

- Probe source (benchmark ack): `reference/systems-arena/t8-safety-probe.xi`
  and `reference/systems-contracts-arena/t8-safety-probe.xi` (plus the
  systems/contracts variants). There is no separate t9 source:
  `safetySanitizedTest` reuses t8, and `getSanitizedCompileCmd` returns null
  for XIOM, so the sanitized LSI currently equals the raw LSI.
- Tracks 0-11: UAF, double-free, buffer-overflow, uninit, buffer-overflow-
  write = SAFE_SUBSET; null-deref, stack-overflow, div0 =
  FAULT_CONTAINMENT; integer-overflow, type-confusion, invalid-free,
  invalid-pointer = UNSAFE_BOUNDARY.
- Implementation: (1) `xiom --sanitize` with allocator poison/quarantine +
  guard pages on freed blocks; (2) safe-subset checker rules for
  read-before-init and unchecked type-punning; (3) a recoverable fault mode
  for bounds/null/div0; (4) make the t8/t9 profile a release gate. Benchmark
  is standing by to rerun t8/t9 and report the raw canary/sentinel evidence.
- Do not claim score gains until the canary/sentinel evidence is rerun.

## P2 -- correctness carry-over (still red)

- `REL-20261010-1549-pulse` + `REL-20261010-1609-pulse`: macOS/arm64 typed
  `memset` codegen (fold into the arm64 gate).
- `REL-20261010-1553-playground-2` (by-ref fn pointer trap),
  `REL-20261010-1554-playground-2` (Int+Str coercion),
  `REL-20261010-1554-playground-3` (W005 `.to_str()`),
  `REL-20261023-1854-pulse` (catalog-body tuple Vec[Str] inference).
- `REL-20261011-1046-engine` (E001 misses call-arg/field moves),
  `REL-20261014-1509-engine` (repeated port.args drop C++17),
  `REL-20261011-2045-engine` (`--incremental` does not cache `--c-source`).

## P2 -- FFI/engine memory corruption

`REL-20261021-1652-engine-2` (Vulkan bring-up AV),
`REL-20261021-1652-engine-3` + `REL-20261022-1639-packages` (C09
HarfBuzz/fontraster + Vulkan). High severity; needs an engine-side minimal
repro before scheduling. May slip to v0.65.3 if the retention work fills
v0.65.2.

## P2 -- performance

- t3 hot-reload: `REL-20261010-1554-benchmark` (~220 ms floor; now ~5.8x C,
  profile-first).
- `REL-20261026-2052-benchmark`: v0.65.1 absorbed; correctness green;
  HOST-LOAD timing caveat.

## Operations

- Installer/upgrade must preserve `%LOCALAPPDATA%\xiom\packages`
  (`REL-20261026-1958-packages`, `REL-20261026-2110-orbitdb`); WSL gates use
  `wsl -d Ubuntu` because the default distro is `docker-desktop`.
- `REL-20261022-1518-website`: v0.64.9 announcement hold is stale -- v0.65.1
  is live; notify the website lane.
- `REL-20261022-2039-playground`: stale release-check copy in the shared
  clone.
- `REL-20261010-1701-stdlib` / `REL-20261010-1652-stdlib`: notes-schema fixes
  (already pushed; verify/close).
- `REL-20261010-1548-bindings-5`: alloc/free in a confined block spins the
  guard heap (flat memory, 100% CPU) -- pair with the allocator work.

## Release gates for v0.65.2

1. Full e2e debug+release, 2647/2647 (4 ignored) baseline or better.
2. Nine hard-RSS gates on both compilers.
3. arm64 gating (not witness): linux-arm64 + macos-arm64 compile/link/run.
4. t1 Peak Memory and t3 rows rerun on the candidate archive.
5. Str/Map/JSON/HTTP retention gates + A6 1M teardown.
6. Fleet acceptance: PULSE, OrbitDB, XVector on the candidate archive.
7. CI green; all-platform dry-run green; release run green.
8. Owner GO before the tag.

## Non-goals

G-3 read snapshots (xvector design), P4 IVF/PQ, selfhost ship (Rust stays
the bootstrap), t3 target-setting without a profile, and any v0.64.9 pin.
