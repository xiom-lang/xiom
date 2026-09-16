// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N14: Derive with method defined -- manually implement method alongside derive
type Vec2 = { x: Int; y: Int; } derive[Eq]
fn Vec2.length(self) -> Int { return self.x * self.x + self.y * self.y; }
fn main() -> Int {
  var a = Vec2{ x: 3; y: 4; };
  var b = Vec2{ x: 3; y: 4; };
  if a == b && a.length() == 25 { return 0; }
  return 1;
}
