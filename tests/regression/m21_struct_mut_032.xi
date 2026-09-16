// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_032
type Pair = { a: Int; b: Int; }

  fn swap_fields(p: Pair) -> Pair {
    var tmp = p.a;
    var result: Pair = p;
    result.a = result.b;
    result.b = tmp;
    return result;
  }

pub fn run() -> Int {
    var p: Pair = { a: 5; b: 9; };
    var swapped = swap_fields(p);
    if swapped.a == 9 && swapped.b == 5 { return 0; }
    return 1;
  }
use m21_struct_mut_032.run;
fn main() -> Int { return run(); }
