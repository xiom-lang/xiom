// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A08: Array in struct -- struct containing an integer field and array ref via index
type Container = { id: Int; val: Int; }
fn main() -> Int {
  var items = [Container{ id: 1; val: 10; }, Container{ id: 2; val: 20; }, Container{ id: 3; val: 30; }];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 3 {
    sum += items[i].val;
    i += 1;
  }
  if sum == 60 { return 0; }
  return 1;
}
