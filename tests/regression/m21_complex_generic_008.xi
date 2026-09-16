// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_008
pub fn wrap_option[T](x: T) -> Option[T] {
    return Some(x);
  }

  pub fn run() -> Int {
    var opt = wrap_option[Int](42);
    match opt {
      Some(v) => if v == 42 { return 0; },
      None => return 1,
    }
  }
use m21_complex_generic_008.run;
fn main() -> Int { return run(); }
