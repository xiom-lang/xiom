# MCU/bare-metal targets (1220/1419)

Status: IN PROGRESS -- v0.65.0 first wave (see docs/RELEASE_GATE_v0.65.0.md
and docs/checklists/v0.65.0.md). The riscv link-triple fix is on main; the
target matrix, no_std profile, startup/linker contract, and QEMU CI are now
in the v0.65.0 release scope.
Owner: compiler lane (targets/runtime), bindings lane (HAL packages)
Sources: `REL-20261019-1220-bindings` (owner request),
`REL-20261019-1419-bindings` (target probe), `docs/SYSTEMS_TRACK_PLAN.md`.

## Current state

- `--target arm` is AArch64 **Linux** (`aarch64-unknown-linux-gnu`), not
  Cortex-M.
- `--target riscv` emitted/links with the GCC arch spelling
  `riscv64gc-unknown-linux-gnu`; clang rejects that triple. The LLVM spelling
  `riscv64-unknown-linux-gnu` is the fix (REL-20261019-1419).
- No `thumbv6m`/`thumbv7em`/`thumbv8m`, no `riscv32imc`/`riscv32imac`, no
  freestanding profile, no startup/linker contract, no QEMU matrix.

## Plan

- [x] Correct the RISC-V link triple to `riscv64-unknown-linux-gnu` and keep
      the IR triple consistent (`crates/xiom/src/lib.rs`; e2e triple lock).
- [ ] Define the target matrix:
      `thumbv6m-none-eabi`, `thumbv7em-none-eabi`,
      `thumbv8m.main-none-eabi` (if cheap), `riscv32imc-unknown-none-elf`,
      `riscv32imac-unknown-none-elf`. Keep the existing `arm`/`riscv`
      desktop/Linux meanings unchanged.
- [ ] Add a freestanding/no_std profile: no CRT, no host libc, core-only
      stdlib surface, panic-handler hook, allocator policy hook, and
      deterministic interrupt-vector story.
- [ ] Define the startup/linker contract: entry point, vector table,
      linker-script template, section placement, stack/heap symbols, and a
      documented "bring your own linker script" path.
- [ ] Emit/link correctly for the first target pair (one Cortex-M, one
      riscv32) and compile a hello-world under QEMU.
- [ ] Add a QEMU CI matrix (micro:bit nRF51 / STM32VLDISCOVERY / netduino
      and a riscv32 machine). Vendor peripherals stay SKIP/hardware for the
      first wave.
- [ ] Coordinate the host-companion packages (firmata/micropython/uf2) and
      the HAL package family with the bindings lane; no package publish
      until a target passes the QEMU gate.

## Acceptance (first wave)

- `xiom --target riscv` reaches clang with a valid triple (no
  `unknown target triple`).
- A minimal freestanding program compiles and runs on one emulated
  Cortex-M board and one emulated riscv32 machine.
- The no_std hello has no host libc/CRT references and a documented panic
  path.
- Desktop, wasm, wasi, arm, and riscv64 behavior is unchanged; existing
  target-triple tests stay green.

## Non-goals (first wave)

- Emulating vendor peripherals or shipping a full HAL.
- Arduino AVR (LGPL core; API reimplementation only if wanted).
- Replacing the existing AArch64 Linux `arm` target.
