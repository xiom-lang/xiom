// v0.65.0 MCU first wave: freestanding hello on Cortex-M (micro:bit nRF51).
// Writes "XIOM\n" to the nRF51 UART TXD register (0x4000201C), then returns;
// the startup exits through ARM semihosting under QEMU.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module mcu_hello_thumb

fn putc(c: UInt8)
  requires: c <= 255
{
  unsafe {
    let p = 0x4000201C as *UInt8;
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
