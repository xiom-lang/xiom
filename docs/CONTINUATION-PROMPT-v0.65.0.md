# Continuation prompt -- XIOM v0.65.0 compiler lane

Paste the block below into the successor session.

```text
You are the compiler-lane successor for XIOM (E:\xiom-lang\xiom). Do not
continue the withdrawn v0.64.9 release; production remains v0.64.8.

Read, in this order:
1. `docs/SESSION-HANDOFF.md` top block (2026-10-22T19:52Z).
2. `docs/RELEASE_GATE_v0.65.0.md`.
3. `docs/checklists/v0.65.0.md` and `docs/checklists/mcu-targets.md`.
Pull `E:\xiom-lang\xiom-relays` first and process new compiler items:
REL-20261022-1529-pulse (ptrtoint ptr @__unsafe_block_N, LLVM 14/Apple
arm64) and REL-20261022-1827-packages (three-probe data-loss cluster).
Also read REL-20261022-1645-packages and REL-20261021-1615-xvector.

State at handoff:
- v0.64.9 is WITHDRAWN (GitHub pre-release); production is v0.64.8.
  Do not pin/deploy v0.64.9. Stdlib v0.64.9 tag/registry exist but are
  held; v0.65.0 will coordinate a new stdlib cut.
- Main `dfd234845` plus the handoff commit. Workspace/selfhost 0.65.0.
  `STDLIB_VERSION` is still `stdlib-v0.64.9` for development.
- Fixed on main: nested enum payloads (m1615/1243, commit ecd22a225,
  e2e `e2e_m1615_nested_enum_payloads`), m372/1451 pointee inference
  (same commit; nested=42 local=42), partial true 1550 (dfd234845: map U
  now tuple; collect still U=Int and em[1].0=0). Earlier wave also on
  main: m1157, 2045 part 2, XVC-C-22, m1731, m1703, m1326, m1520, 1511
  package.xi deps alias, m365 dependency-key split, 1531 dead-builder
  lock, m375 opt-in XIOM_FAST_EXIT.
- Battery on current main: feature 549/549, integration 130/130, stdlib
  40/40, codegen lib 25/25. Re-run after each change.

Goal (start a new session Goal with this objective):
Deliver XIOM v0.65.0 as a production release, no rush, per
docs/RELEASE_GATE_v0.65.0.md. P0:
1. Finish true 1550: `iter.range(10,13).enumerate().map(idx_plus).collect()`
   must return em=[(0,10),(1,12),(2,14)]. `collect` still specializes as
   `MapIter.collect_Tuple__Int__Int_Int`; resolve the map call's returned
   `MapIter[(Int, Int), U]` for the collect receiver, then add the true
   `em[1].0` e2e lock.
2. Fix the 1827/1645/2045 data-loss cluster and add all three probes to
   the test set: direct `Err(E.Bad(Str))` pushed into Vec (loses Str;
   local-bound control works), `Vec[Int]` inside a cross-module
   `Result[(Nominal, Vec[Int]), Str]` (contents lost at the module
   boundary), and direct `Err(Enum.Variant(..))` match. Gate release-mode
   codegen on them.
3. Fix 1529: stored-fn env thunks emit `ptrtoint ptr @__unsafe_block_N`;
   emit a typed `i8*` operand so clang 14 / Apple arm64 accepts it. Add
   the stored-fn probe to the arm64 floor gate.
4. Add release-compiler parity: build `target/release/xiom.exe` and run
   the P0 probes; add a CI job. This is the gate v0.64.9 missed.
5. m1511: make dependency `port.args.json` a transitive, cycle-safe
   closure; add a two-level installed-layout test (consumer -> fontraster
   -> harfbuzz) and an installed fontraster build.
6. C07: reproduce the v0.64.9 capture-probe memory regression (1.43-1.50
   GB vs v0.64.4) and fix or formally scope it.

P1: full debug+release battery, negative/ASAN/reproducibility tests,
reporter-lane verification on the release archive. P2: MCU first wave per
docs/checklists/mcu-targets.md: thumbv6m/thumbv7em/thumbv8m.main,
riscv32imc/imac, `--freestanding` no_std, startup/linker contract,
Cortex-M + riscv32 hello under QEMU, QEMU CI, desktop/wasm/wasi/arm/
riscv64 unchanged. P3: version 0.65.0, stdlib v0.65.0 first (pin v0.64.8),
merged notes, all-platform dry-run, owner GO, tag v0.65.0, 11 assets,
registry xiom-std 0.65.0, announcements. No tag/candidate before green.

Critical constraints:
- Do not pin/deploy v0.64.9; it is a pre-release/audit artifact.
- Cargo is at C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin
  (the engine incident deleted .cargo); prepend it to PATH for cargo/rustc.
- No stdlib edits except via relay; the stdlib lane cuts its release first.
- Selfhost branch `selfhost-phase-4-codegen` is local (tip 3e9f26484);
  RD11 T3 is green (19/19, 14/64, 83/83), diff_ast 80/83. The trailing
  match tail re-wrap reaches 83/83 but needs a value-returning
  NkExprMatch codegen path; do not enable the parser half alone.
- m375 1M A/B and m365 117-target re-measure are post-release lane rows.
- Test commands: cargo test -p xiom-codegen --test
  {feature_regression_tests,integration_tests,stdlib_tests,e2e_tests};
  cargo test -p xiom-codegen --lib; cargo test -p xiom --lib;
  cargo test -p xiom-check --lib; cargo test -p xiom-parser;
  cargo test -p xiom-graph.
- Fixtures: tests/regression/m1615_nested_enum_payloads.xi,
  m372_nested_generic_ptr_read.xi, m1520_adapter_collision.xi,
  m1550_enumerate_tuple_map.xi; true 1550:
  E:\xiom-lang\stdlib\tools\known_failures\p_iter_enumerate_tuple.xi;
  packages 1827 probes under E:\xiom-packages\packages\packages\.
```
