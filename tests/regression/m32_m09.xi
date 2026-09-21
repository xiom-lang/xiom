// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M09: Nested modules -- modules declared inside modules
module outer {
  pub fn val_a() -> Int { return 10; }
  module inner {
    pub fn val_b() -> Int { return 20; }
  }
}
use outer.val_a;
use outer.inner.val_b;
fn main() -> Int {
  var a = val_a();
  var b = val_b();
  if a == 10 && b == 20 { return 0; }
  return 1;
}
