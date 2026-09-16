// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z16: Compound assignment with cast
fn main() -> Int {
  var x: Int32 = 100;
  var y: Int64 = 200;
  x += 50;
  y -= 75;
  x *= 3;
  y /= 5;
  var result: Int = (x as Int) + (y as Int);
  if result == 475 { return 0; }
  return 1;
}
