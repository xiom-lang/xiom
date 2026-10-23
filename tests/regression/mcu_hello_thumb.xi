// v0.65.0 MCU first wave: freestanding hello on Cortex-M (micro:bit nRF51).
// Prints "XIOM\n" through ARM semihosting (SYS_WRITEC); QEMU's microbit
// machine does not route the nRF51 UART to stdio. The startup exits through
// semihosting with the program's status.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module mcu_hello_thumb

extern "C" {
  fn mcu_putc(c: UInt8) -> Int;
}

fn putc(c: UInt8)
  requires: c <= 255
{
  unsafe {
    mcu_putc(c);
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
