// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_007
pub fn run() -> Int
  requires: true
{
    unsafe {
      var x: Int32 = 1000i32;
      var p: *Int32 = &x;
      if *p == 1000i32 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_007.run;
fn main() -> Int { return run(); }
