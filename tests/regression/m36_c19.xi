// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C19: Every module nesting depth 1-5 -- deeply nested modules with unique pub function names
module a1 {
  pub fn val1() -> Int { return 1; }
  module a2 {
    pub fn val2() -> Int { return 2; }
    module a3 {
      pub fn val3() -> Int { return 3; }
      module a4 {
        pub fn val4() -> Int { return 4; }
        module a5 {
          pub fn val5() -> Int { return 5; }
        }
      }
    }
  }
}
module b1 {
  pub fn double(x: Int) -> Int { return x * 2; }
  module b2 {
    pub fn triple(x: Int) -> Int { return x * 3; }
    module b3 {
      pub fn quad(x: Int) -> Int { return x * 4; }
    }
  }
}
use a1.val1;
use a1.a2.val2;
use a1.a2.a3.val3;
use a1.a2.a3.a4.val4;
use a1.a2.a3.a4.a5.val5;
use b1.double;
use b1.b2.triple;
use b1.b2.b3.quad;
fn main() -> Int {
  if val1() != 1 { return 1; }
  if val2() != 2 { return 2; }
  if val3() != 3 { return 3; }
  if val4() != 4 { return 4; }
  if val5() != 5 { return 5; }
  var sum = val1() + val2() + val3() + val4() + val5();
  if sum != 15 { return 6; }
  if double(21) != 42 { return 7; }
  if triple(7) != 21 { return 8; }
  if quad(5) != 20 { return 9; }
  return 0;
}
