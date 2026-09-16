// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_003
pub fn run() -> Int
  requires: true
{
    unsafe {
      var x: Int = 100;
      var p: *Int = &x;
      var y: Int = *p;
      if y == 100 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_003.run;
fn main() -> Int { return run(); }
