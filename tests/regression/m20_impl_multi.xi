// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

interface A { fn a(self) -> Int; }
interface B { fn b(self) -> Str; }
type V = { val: Int; }
impl A for V {
  fn a(self) -> Int { return self.val; }
}
impl B for V {
  fn b(self) -> Str { return "ok"; }
}
fn main() -> Int {
  var v = V{ val: 42 };
  if v.a() != 42 { return 1; }
  if v.b() != "ok" { return 2; }
  return 0;
}