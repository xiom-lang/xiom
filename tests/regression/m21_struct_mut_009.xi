// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_009
type Container = { items: Vec[Int]; label: Int; }
fn main() -> Int {
  var c: Container = Container{ items: Vec[Int].new(); label: 0; };
  c.items.push(1); c.items.push(2); c.items.push(3);
  c.label = 7;
  if c.items.len() == 3 && c.label == 7 { return 0; }
  return 1;
}
