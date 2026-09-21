// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module extern_lib
extern "C" {
  fn puts(s: *UInt8) -> Int;
}
pub fn greet() -> Int {
  return puts("hello");
}
