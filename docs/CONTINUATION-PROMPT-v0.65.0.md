# Continuation prompt -- XIOM v0.65.0 compiler lane

Paste the block below into the successor session.

```text
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom). Do not
continue the withdrawn v0.64.9 release; production remains v0.64.8 until
the v0.65.0 tag.

Read, in this order:
1. `docs/SESSION-HANDOFF.md` top block (2026-10-23T05:00Z).
2. `docs/RELEASE_GATE_v0.65.0.md` (P3 blocking dependencies).
3. `docs/checklists/v0.65.0.md`, `docs/checklists/mcu-targets.md`,
   `docs/MCU_FREESTANDING.md`.
Pull `E:\xiom-lang\xiom-relays` first; the compiler queue was processed
through REL-20261023-0423-compiler. Watch for stdlib's reply to
REL-20261023-0242-compiler (the C07 runtime helper) and the stdlib v0.65.0
tag.

State at handoff (2026-10-23T05:00Z):
- v0.64.9 is WITHDRAWN (GitHub pre-release); production is v0.64.8.
  Do not pin/deploy v0.64.9.
- Main `ade98fb68`. Workspace/Cargo.lock 0.65.0. `STDLIB_VERSION` is
  still `stdlib-v0.64.9`; change it to `stdlib-v0.65.0` only after the
  stdlib cut.
- P0 complete on main: true 1550 (dd9256343; em=[(0,10),(1,12),(2,14)],
  em[1].0==1), 1827/1645/2045 (45804b195, narrowed 44dcf01c0; 3 e2e
  locks), 1529 (a6ba113ed; typed ptrtoint, arm64 floor), release parity
  (4acd28bd7 + CI), m1511 transitive port args (bf4709a5a; installed
  fontraster build), C07 (44dcf01c0; engine probe PASS 821 MB vs 1391 MB
  killed). Battery regressions m44/m48/m35_t21/parallel-codegen closed in
  44dcf01c0.
- P1: full battery + full e2e 2646/2646 (4 ignored) on debug AND release;
  ASAN clean on the P0 probes; IR/exe reproducible. Lane re-verification
  happens on the v0.65.0 archive.
- P2 MCU first wave (2c5a2192d): thumbv6m/thumbv7em/thumbv8m,
  riscv32imc/riscv32imac, --freestanding, startup/linker/runtime contract,
  BYO --linker-script, QEMU CI mcu-qemu. Both hellos link with zero
  undefined symbols.
- P3 remains:
  1. stdlib v0.65.0 must include `xiom_guard_copy_out_if_arena` (see
     REL-20261023-0242-compiler) and a v0.65.0 notes fragment. Then pin
     STDLIB_VERSION=stdlib-v0.65.0, regenerate
     `release-notes/v0.65.0.json` (cargo run -p xiom-release-notes --
     convert --tag v0.65.0 --stdlib ../stdlib), and re-run the release
     battery.
  2. All-platform dry-run on the release commit (Windows/Linux x64+arm64,
     macOS x64+arm64, wasm, VSIX) and the QEMU/release jobs.
  3. Owner GO; then annotated tag v0.65.0, 11 assets + SHA256SUMS, docs
     dispatch, registry xiom-std 0.65.0, lane announcements, holds closed.
- Release notes are authored at release-notes/v0.65.0.md; the JSON must be
  regenerated after the stdlib fragment. Packaging ships
  runtime/freestanding under lib/runtime/freestanding.

Critical constraints:
- Cargo is at C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin
  (the engine incident deleted .cargo); prepend it to PATH.
- No stdlib edits except via relay; the stdlib lane cuts its release first.
- Selfhost branch `selfhost-phase-4-codegen` is local (tip 3e9f26484);
  RD11 T3 green, diff_ast 80/83. The trailing match tail re-wrap needs a
  value-returning NkExprMatch codegen path; do not enable the parser half
  alone. m375 1M A/B and m365 117-target re-measure are post-release lane
  rows.
- Test commands: cargo test -p xiom-codegen --test
  {feature_regression_tests,integration_tests,stdlib_tests,e2e_tests};
  cargo test -p xiom-codegen --lib; cargo test -p xiom --lib;
  cargo test -p xiom-check --lib; cargo test -p xiom-parser --lib;
  cargo test -p xiom-graph --lib.
- Release-mode parity: set XIOM_TEST_COMPILER=target/release/xiom(.exe)
  and XIOM_STDLIB=<repo>/stdlib; the release-parity CI job does the same.
```
