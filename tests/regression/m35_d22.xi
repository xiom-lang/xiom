// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D22: Topological sort -- in-degree calculation pattern
fn main() -> Int {
  var d0: Int = 0;
  var d1: Int = 2;
  var d2: Int = 1;
  var d3: Int = 3;
  if d0 != 0 { return 1; }
  if d1 != 2 { return 2; }
  if d2 != 1 { return 3; }
  if d3 != 3 { return 4; }
  return 0;
}
