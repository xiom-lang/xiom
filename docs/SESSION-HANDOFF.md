<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-11 (compiler lane, context-full checkpoint)

This file captures state for continuing the 2026-10-11 batch program in a
fresh session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Landed this session (all unpushed, next batch)

- `b443e970` m254 tuple-literal indexed element (pre-existing, carried)
- `13115d64` m255 bare-filename check indexes the file's own dir (C-ORBIT-06
  reconciliation)
- `458d0afb` m256 Vec-arm-binding tuple return (selfhost clang failure)
- `9af8434f` m257 `--icon` in `--help`
- `54a9a619` m258 `--cxx-standard` for C++ `--c-source` TUs
- `6cbcc53e` 1854 pulse startup-AV diagnosis (OPEN item, docs)
- `186b325f` rustc reference index; `ecc587ee` clone extents;
  `a62e8f14` rustc study program (7 docs + intake + planning folds)
- `9bd05558` m259 W004 body-taint gating (1900 fixed)
- `d4907d48` m260 [N]UInt8 element reads zero-extend (C-ORBIT-08 B fixed)
- Bus: pushed through `5e027f5` (1900 fix, 1219 ack, batch-plan item
  `REL-20261011-1342-compiler` to all, stdlib pin item
  `REL-20261011-1342-compiler-2`).

## 2. Six batches (relay item REL-20261011-1342-compiler has the full text)

1. RESOLVER/ISOLATION (1052+1124, both HIGH) -- FIRST.
2. FN-VALUE/CALLBACK ABI (1904, 1055/B-13, 1121/B-14-15, 0317).
3. PHANTOM IMPORTS (1802 core, 2124).
4. TOOLCHAIN HOME/PATH (2100, 2102) -- packages + registry.
5. PULSE STARTUP AV (1854, 2122).
6. FLOAT32 SLOT STORES (1134).

Gates and witnesses per batch are in that relay item; the stdlib PIN
coordination is REL-20261011-1342-compiler-2 (numeric witness:
metrohash64(empty,0) == 1424585646250073642, wrong today
15427970114922222902).

## 3. Repro / witness map

- Resolver hijack: `E:\xiom-projects\xiom-orbitdb\docs\repro\consumer-fn-hijack\`
  (mod_svc.xi + probe_hijack.xi + probe_clean.xi; expect
  `svc: private arm() ran`, both import orders).
- Private-symbol collision: stdlib `xiom/hash/{farm,metro,city}.xi`;
  co-import probe with the numeric witness above.
- Enum-in-tuple (1219 part A, OPEN): orbitdb
  `docs/repro/enum-tuple-codegen/repro_enum_tuple.xi`; minimal shape is
  `Result[(Color, Int), Int]` with a payload enum (plain tuple is green;
  nondeterministic STATUS_BREAKPOINT 0x80000003).
- Pulse AV: `E:\xiom-lang\xiom` docs section "1854" + relay 2122 (bisected
  to the `&HandlerOut` helper; unused-fn emission cascade 341KB->1.9MB and
  two `movq 0xf8` TLS-offset loads; studies 02/03 for direction).
- Float32 stores: xvector `docs/repro/xv-f32-slot-literal-write/probe.xi`.
- u8 read sibling: fixed m260 (`tests/regression/m260_u8_array_elem_cast/`).
- m259 lock: `tests/regression/m259_w004_no_cascade/` +
  `checker_locks.rs::m259_w004_suppressed_on_error_tainted_body`.

## 4. Environment / ops notes

- Compiler repo: local main is AHEAD of origin by the stack above; keep
  commits unpushed unless the doorbell's next-batch push says otherwise.
- Bus: `E:\xiom-lang\xiom-relays`; `relay.py new --body-file` avoids
  PowerShell quote-splitting; note strings must avoid embedded `"` when
  passed as args (use --body-file). Items to `all` cannot be acked by a
  single lane (relay enforces).
- Dev-box quirks: `cargo test -p xiom --lib` spins on test_compile_* (pre-
  existing box condition; reproduces pre-m255) -- use filtered
  e2e/checker_locks runs; MCP/xiom servers and other lanes share the box.
- Windows Defender blocks some %TEMP% exe paths (os error 225 class);
  keep probes under tests/regression or lane scratch.
- Agent Manager: 7 `study/rustc-*` worktrees exist (docs already landed
  into main); selfhost phase worktrees exist (phases 1-4). Do not touch
  other lanes' worktrees.
- Ops rule (replayed from REL-20261011-1134-ops): shared toolchain homes
  are read-only; no destructive install/uninstall tests; announce any
  destructive window to: all first.
- m-numbering: m261-m267 LANDED (2026-10-11, batch 1-3 commits); next free is **m268**.

## 5. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), continuing the 2026-10-11
batch program. Read docs/SESSION-HANDOFF.md FIRST; consult docs/RUSTC-STUDY.md
and docs/rustc-study/ for reference patterns; rustc source is E:\repos\rust
(READ-ONLY; no git writes there).

START BATCH 1 -- resolver/isolation (REL-20261011-1052-stdlib +
REL-20261011-1124-orbitdb, both HIGH; plan text in the to-all item
REL-20261011-1342-compiler):
1) Reproduce the hijack: copy
   E:\xiom-projects\xiom-orbitdb\docs\repro\consumer-fn-hijack\* to a clean
   temp dir; run target\debug\xiom.exe on mod_svc + probe_hijack (path form,
   not bare filenames); confirm the consumer's arm(Str,Str,Str) runs instead
   of svc's private arm().
2) Reproduce the numeric witness: probe co-importing xiom.hash.farm +
   xiom.hash.metro (from E:\xiom-lang\stdlib) shows metrohash64(empty,0) ==
   15427970114922222902 (wrong); correct value 1424585646250073642; check
   both import orders.
3) Write failing fixtures + locks: tests/regression/m261_module_local_precedence/
   (hijack shape; expect the svc-private path) and
   tests/regression/m262_private_symbol_isolation/ (farm+metro numeric pin,
   BOTH import orders); add e2e fns + the ci.yml list entry (mirror the
   e2e_m256/e2e_m260 pattern).
4) Fix: audit unqualified-call resolution (crates/xiom-check +
   crates/xiom-codegen): module-local precedence (caller's own module first,
   incl. privates) and module-private symbol identity (per-module mangling
   OR duplicate-definition rejection; rustc anchors: rustc_resolve,
   rustc_privacy, rustc_symbol_mangling). Extend the m225 consumer-scope
   machinery; keep the m225 set green.
5) Verify: both witnesses green in both orders; run the scope/array e2e
   regression sets; commit as m261/m262 (next free numbers), with
   docs/COMPILER_BUGS.md sections; relay: ack+fix BOTH items with the
   numeric witness as evidence (use --body-file for notes).

THEN BATCH 2 -- fn-value/callback ABI (1904, 1055/B-13, 1121/B-14-15,
0317-pulse-7): root the shapes from docs/rustc-study/05 (fn-ptr signature
recorded at binding, return mode derived at call emission); marshal paths
(f as Int / re-pass / struct field / local) must keep the address; Vec[fn]
element calls of any arity must type their declared return; Float64 through
cast fn-ptrs must read the float register. Fixtures from the bindings
probes; same lock/commit/bus discipline.

THEN batches 3-6 in order (phantom imports 1802+2124; toolchain home/path
2100+2102; pulse AV 1854+2122; Float32 stores 1134) -- gates in
REL-20261011-1342-compiler.

RULES: workarounds stay until each fix ships and is re-verified from its
witness; ONE commit per batch (fix + fixtures + docs); keep the stack
unpushed for the doorbell's next-batch push; notes/commit messages in the
house voice (no fluff); ASCII only; no destructive toolchain tests; local
lib-suite spin is a known box condition -- use filtered runs.
```
