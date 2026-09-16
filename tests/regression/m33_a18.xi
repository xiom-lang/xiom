// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A18: Array comparison -- element-by-element comparison of two arrays
fn main() -> Int {
  var a = [1, 2, 3, 4, 5];
  var b = [1, 2, 3, 4, 5];
  var c = [1, 2, 3, 4, 6];
  var eq_ab: Bool = true;
  var eq_ac: Bool = true;
  var i: Int = 0;
  while i < 5 {
    if a[i] != b[i] { eq_ab = false; }
    if a[i] != c[i] { eq_ac = false; }
    i += 1;
  }
  if eq_ab && !eq_ac { return 0; }
  return 1;
}
