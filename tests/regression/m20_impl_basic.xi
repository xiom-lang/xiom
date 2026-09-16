// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

interface Show { fn show(self) -> Str; }
type Point = { x: Int; y: Int; }
impl Show for Point {
  fn show(self) -> Str {
    return "(" + self.x.to_string() + "," + self.y.to_string() + ")";
  }
}
fn main() -> Int {
  var p = Point{ x: 3, y: 4 };
  var s = p.show();
  if s == "(3,4)" { return 0; }
  return 1;
}