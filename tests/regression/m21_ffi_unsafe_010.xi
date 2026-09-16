// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_010
extern "C" {
    fn strlen(s: *UInt8) -> Int;
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_ffi_unsafe_010.run;
fn main() -> Int { return run(); }
