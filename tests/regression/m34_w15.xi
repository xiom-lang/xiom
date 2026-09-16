// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W15: Right-shift sign extension -- arithmetic shift on signed Int
fn main() -> Int {
  var neg: Int = -256;
  var pos: Int = 256;
  var r_neg4: Int = neg >> 4;
  var r_pos4: Int = pos >> 4;
  var r_neg8: Int = neg >> 8;
  if r_neg4 == -16 && r_pos4 == 16 && r_neg8 == -1 { return 0; }
  return 1;
}
