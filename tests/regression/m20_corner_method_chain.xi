// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

type P = { x: Int; y: Int; }
fn P.norm(self) -> Int { return self.x + self.y; }
fn main() -> Int { var p = P{ x: 3, y: 4 }; if p.norm() != 7 { return 1; } return 0; }