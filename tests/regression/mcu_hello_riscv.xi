// v0.65.0 MCU first wave: freestanding hello on riscv32 (QEMU `virt`).
// Writes "XIOM\n" to the 16550 UART THR at 0x10000000; the startup writes
// the SiFive test finisher so QEMU exits with the program's status.
// QEMU loads the raw binary at 0x80200000 (`-kernel out/mcu_riscv.bin`).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module mcu_hello_riscv

fn putc(c: UInt8)
  requires: c <= 255
{
  unsafe {
    let p = 0x10000000 as *UInt8;
    *p = c;
  }
}

pub fn main() -> Int {
  putc(88 as UInt8);  // X
  putc(73 as UInt8);  // I
  putc(79 as UInt8);  // O
  putc(77 as UInt8);  // M
  putc(10 as UInt8);  // newline
  return 0;
}
