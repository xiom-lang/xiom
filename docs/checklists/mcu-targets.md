# MCU/bare-metal targets (1220/1419)

Status: IN PROGRESS -- v0.65.0 first wave (see docs/RELEASE_GATE_v0.65.0.md
and docs/checklists/v0.65.0.md). The riscv link-triple fix is on main; the
target matrix, no_std profile, startup/linker contract, and QEMU CI are now
in the v0.65.0 release scope.
Owner: compiler lane (targets/runtime), bindings lane (HAL packages)
Sources: `REL-20261019-1220-bindings` (owner request),
`REL-20261019-1419-bindings` (target probe), `docs/SYSTEMS_TRACK_PLAN.md`.

## Current state

- `--target arm` remains AArch64 **Linux** (`aarch64-unknown-linux-gnu`);
  `--target riscv` remains riscv64 Linux.
- The v0.65.0 first wave adds `thumbv6m`, `thumbv7em`, `thumbv8m`,
  `riscv32imc`, and `riscv32imac` as freestanding targets with the startup /
  linker / runtime contract in `docs/MCU_FREESTANDING.md`.
- The riscv32 triple is `riscv32-unknown-none-elf` with `-march=rv32imc` /
  `rv32imac`; clang rejects an ISA suffix in the triple itself.
- QEMU execution is gated in CI (`mcu-qemu`); vendor peripherals are not
  emulated in the first wave.

## Plan

- [x] Correct the RISC-V link triple to `riscv64-unknown-linux-gnu` and keep
      the IR triple consistent (`crates/xiom/src/lib.rs`; e2e triple lock).
- [x] Define the target matrix:
      `thumbv6m-none-eabi`, `thumbv7em-none-eabi`,
      `thumbv8m.main-none-eabi`, `riscv32imc` / `riscv32imac`
      (`riscv32-unknown-none-elf` + `-march`; clang rejects the ISA suffix in
      the triple). Keep the existing `arm`/`riscv` desktop/Linux meanings
      unchanged. e2e `e2e_mcu_target_triples`.
- [x] Add a freestanding/no_std profile (`--freestanding`, implied by MCU
      targets): no CRT/host libc, plain (non-TLS) recursion counter, bump
      allocator, no-op guard heap/page, trampoline ABI, `xiom_panic` hook.
      See `docs/MCU_FREESTANDING.md`.
- [x] Define the startup/linker contract: entry point, vector table,
      linker-script templates (`runtime/freestanding/{cortex_m,riscv32}.ld`),
      section placement, stack/heap symbols, and `--linker-script` for a
      bring-your-own script.
- [x] Emit/link correctly for the first target pair (thumbv6m + riscv32imac);
      both hellos link with `-nostdlib` and zero undefined symbols. e2e
      `e2e_mcu_freestanding_links`.
- [x] Add a QEMU CI matrix: `mcu-qemu` runs the thumbv6m hello on
      `-M microbit -semihosting` and the riscv32imac hello on
      `-M virt -bios none`, asserting `XIOM` output. thumbv7em/thumbv8m/
      riscv32imc stay link-gated; vendor peripherals SKIP/hardware.
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
