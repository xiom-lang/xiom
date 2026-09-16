// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-18: Deep tuple-like nesting via layered struct pairs -- 10-level chain
type P1 = { x: Int; y: Int; }
type P2 = { a: P1; b: P1; }
type P3 = { l: P2; r: P2; }
type P4 = { p: P3; }

fn main() -> Int {
  var s = P4{ p: P3{ l: P2{ a: P1{ x: 1; y: 2; }; b: P1{ x: 3; y: 4; }; };
                      r: P2{ a: P1{ x: 5; y: 6; }; b: P1{ x: 7; y: 8; }; }; }; };
  if s.p.l.a.x == 1 && s.p.l.a.y == 2 &&
     s.p.l.b.x == 3 && s.p.l.b.y == 4 &&
     s.p.r.a.x == 5 && s.p.r.a.y == 6 &&
     s.p.r.b.x == 7 && s.p.r.b.y == 8 { return 0; }
  return 1;
}
