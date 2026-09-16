// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_008
pub fn run() -> Int
  requires: true
{
    unsafe {
      var arr: Vec[Int] = [10, 20, 30];
      var p: *Int = &arr[0];
      if *p == 10 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_008.run;
fn main() -> Int { return run(); }
