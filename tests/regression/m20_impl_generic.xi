// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

interface Describe { fn desc(self) -> Str; }
type Box = { val: Int; }
impl Describe for Box {
  fn desc(self) -> Str { return "Box: " + self.val.to_string(); }
}
fn main() -> Int {
  var b = Box{ val: 7 };
  var s = b.desc();
  if s == "Box: 7" { return 0; }
  return 1;
}