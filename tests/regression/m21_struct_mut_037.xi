// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_037
type Box[T] = { val: T; }

pub fn run() -> Int {
    var b: Box[Int] = { val: 10; };
    b.val = 42;
    if b.val == 42 { return 0; }
    return 1;
  }
use m21_struct_mut_037.run;
fn main() -> Int { return run(); }
