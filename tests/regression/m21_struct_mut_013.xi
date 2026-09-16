// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_013
type Container = { data: Int; }
fn main() -> Int {
  var c: Container = Container{ data: 5; };
  c.data = c.data + 3;
  c.data = c.data * 2;
  if c.data == 16 { return 0; }
  return 1;
}
