// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D30: Combined stress -- stack + heap integration
fn main() -> Int {
  var s0: Int = 10;
  var s1: Int = 20;
  var h0: Int = 10;
  var h1: Int = 20;
  var h2: Int = 30;
  if s0 != 10 { return 1; }
  if s1 != 20 { return 2; }
  if h0 > h1 { return 3; }
  if h1 > h2 { return 4; }
  if h0 < h1 { h0 = h0; }
  return 0;
}
