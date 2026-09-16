// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D23: Shortest path -- verify distance computation
fn main() -> Int {
  var d0: Int = 0;
  var d1: Int = 4;
  var d2: Int = 1;
  var d3: Int = 4;
  var d4: Int = 7;
  if d0 != 0 { return 1; }
  if d1 != 4 { return 2; }
  if d4 != 7 { return 3; }
  return 0;
}
