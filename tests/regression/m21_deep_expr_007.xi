// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_007
type Num = { val: Int; }

  fn Num.add(n: Int) {
    self.val = self.val + n;
  }

  pub fn run() -> Int {
    var n: Num = { val: 1; };
    n.add(1);
    n.add(2);
    n.add(3);
    n.add(4);
    n.add(5);
    if n.val == 16 { return 0; }
    return 1;
  }
use m21_deep_expr_007.run;
fn main() -> Int { return run(); }
