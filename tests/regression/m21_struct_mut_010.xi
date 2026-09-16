// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_010
type Holder = { data: Vec[Int]; sum: Int; }
fn main() -> Int {
  var h: Holder = Holder{ data: Vec[Int].new(); sum: 0; };
  h.data.push(10); h.data.push(20);
  h.sum = h.data.len();
  if h.sum == 2 && h.data[0] == 10 { return 0; }
  return 1;
}
