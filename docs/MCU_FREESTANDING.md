# MCU / freestanding targets (v0.65.0 first wave)

Status: first wave implemented (target matrix, no_std link profile, startup /
linker contract, QEMU CI). Owner: compiler lane (targets/runtime); HAL and
host-companion packages remain with the bindings lane.

## Target matrix

| CLI `--target` | LLVM triple | ISA / notes |
|---|---|---|
| `thumbv6m` | `thumbv6m-none-eabi` | Cortex-M0/M0+, micro:bit nRF51 |
| `thumbv7em` | `thumbv7em-none-eabi` | Cortex-M4F (link-only in the first wave) |
| `thumbv8m` / `thumbv8m.main` | `thumbv8m.main-none-eabi` | Cortex-M33 (link-only in the first wave) |
| `riscv32imc` | `riscv32-unknown-none-elf` + `-march=rv32imc -mabi=ilp32` | QEMU `virt` |
| `riscv32imac` | `riscv32-unknown-none-elf` + `-march=rv32imac -mabi=ilp32` | QEMU `virt` |

`arm` (AArch64 Linux) and `riscv` (riscv64 Linux) keep their existing meanings
unchanged. `--freestanding` is implied by every target in the table above.

> clang's RISC-V triple parser does not accept an ISA suffix in the triple
> (`riscv32imac-unknown-none-elf` is rejected), so the ISA is selected with
> `-march` and the triple is the generic `riscv32-unknown-none-elf`.

## Freestanding profile

`--freestanding` (or any MCU target) changes three things:

1. **IR/target**: the emitted `target triple` is the bare-metal triple, and
   the recursion-depth counter is a plain `internal global i64` (no TLS;
   bare-metal links have no `__aeabi_read_tp` / TLS segment).
2. **Link**: `clang --target=<triple> -fuse-ld=lld -nostdlib -ffreestanding
   -fno-builtin -fno-unwind-tables -fno-asynchronous-unwind-tables
   -Wl,--gc-sections -Wl,-T,<script>`, plus the arch startup and
   `runtime/freestanding/bare_runtime.c`. No host CRT, libc, or sanitizer
   runtime is linked.
3. **Runtime**: a bump allocator (`xiom_alloc` / `xiom_guard_alloc`), no-op
   guard-heap/guard-page hooks, no-op Copy-Out (there is no arena), the
   unsafe-block trampoline ABI, `xiom_panic`, and `memcpy`/`memset`/
   `memmove`/`strlen`.

## Startup / linker contract

Files live in `runtime/freestanding/` (compiler repo; shipped with the
compiler, not the stdlib):

| File | Contract |
|---|---|
| `cortex_m_startup.c` | `.vectors` at 0x0: initial SP from `_estack`, `Reset_Handler`, default handlers. `Reset_Handler` copies `.data` from `__data_load`, zeroes `.bss`, calls `main`, then exits via ARM semihosting (`bkpt 0xAB`); without a debugger it stays in the trap loop. |
| `riscv32_startup.c` | `_start` sets `sp = _estack`, zeroes `.bss` (the recursion counter and bump pointer live there), calls `main`, writes the SiFive test finisher at `0x00100000` (`(rc << 16) | 0x5555`) so QEMU exits with the program status. |
| `cortex_m.ld` | ENTRY(Reset_Handler); FLASH 0x0/256K, RAM 0x20000000/16K (micro:bit nRF51). Defines `__data_load/start/end`, `__bss_start/end`, `__heap_start/end`, `_estack`. |
| `riscv32.ld` | ENTRY(_start); RAM 0x80200000/16M (QEMU `virt` kernel load address; 0x80000000 is firmware space). Same symbols. |
| `bare_runtime.c` | The no_std runtime ABI listed above; `xiom_panic` is weak so a board can override it. Cortex-M also exports `mcu_putc` (semihosting `SYS_WRITEC`) for the hello. |

**Bring your own linker script**: pass `--linker-script <path>`. The script
must define `_estack`, `__heap_start`, `__heap_end`, and (Cortex-M)
`__data_load`, `__data_start`, `__data_end`, `__bss_start`, `__bss_end`.
The vector table must be placed at the SoC's boot address.

**Panic path**: `xiom_panic` traps in an infinite loop by default. A board
package may provide a strong `xiom_panic` that resets the SoC or writes a
UART message; no unwinding is available.

## Hello programs

- `tests/regression/mcu_hello_thumb.xi` prints `XIOM\n` through ARM
  semihosting (`mcu_putc` -> `SYS_WRITEC`) because QEMU's micro:bit machine
  does not route the nRF51 UART to stdio.
- `tests/regression/mcu_hello_riscv.xi` writes `XIOM\n` to the 16550 UART THR
  at `0x10000000`.

Both build with `xiom --target <target> --release -o hello.elf <fixture>`.

## QEMU gate

CI job `mcu-qemu` (ubuntu-latest):

```sh
qemu-system-arm -M microbit -kernel out/mcu_thumb.elf -nographic -semihosting
qemu-system-riscv32 -M virt -bios none -kernel out/mcu_riscv.elf -nographic
```

Both must print `XIOM` and exit 0. `thumbv7em`, `thumbv8m`, and `riscv32imc`
are link-gated in the same job family; vendor peripherals stay SKIP/hardware
for the first wave.

## First-wave limits

- No general-purpose free: the bump allocator never reuses memory. A HAL or
  application can replace `bare_runtime.c`.
- No exceptions/unwinding; `.ARM.exidx` personality stubs are no-ops.
- No vendor peripheral emulation; the hello uses raw MMIO.
- The stdlib I/O/filesystem surface is host-only; a no_std program should use
  raw pointers or a HAL package.
- HAL/host-companion packages (firmata, micropython, uf2) are bindings-lane
  work and are not published until a target passes the QEMU gate.
