// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B03: Multiple read borrows -- several & references to structs coexist
type Item = { n: Int; }
fn get_n(a: &Item, b: &Item, c: &Item) -> Int { return a.n + b.n + c.n; }
fn main() -> Int {
  var x = Item{ n: 10; };
  var y = Item{ n: 20; };
  var z = Item{ n: 30; };
  var result = get_n(&x, &y, &z);
  if result == 60 && x.n == 10 && y.n == 20 && z.n == 30 { return 0; }
  return 1;
}
