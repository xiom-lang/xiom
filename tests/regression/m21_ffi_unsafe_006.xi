// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_006
pub fn run() -> Int
  requires: true
{
    unsafe {
      var a: Int = 5;
      var b: Int = 10;
      var pa: *Int = &a;
      var pb: *Int = &b;
      if *pa + *pb == 15 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_006.run;
fn main() -> Int { return run(); }
