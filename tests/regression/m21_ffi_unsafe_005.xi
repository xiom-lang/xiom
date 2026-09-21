// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_005
pub fn run() -> Int
  requires: true
{
    unsafe {
      var x: Int8 = 10i8;
      var p: *Int8 = &x;
      if *p == 10i8 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_005.run;
fn main() -> Int { return run(); }
