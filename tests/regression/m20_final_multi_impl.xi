// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

interface A { fn a(self) -> Int; }
interface B { fn b(self) -> Int; }
type X = { x: Int; }
impl A for X { fn a(self) -> Int { return self.x; } }
impl B for X { fn b(self) -> Int { return self.x * 2; } }
fn main() -> Int { var x = X{ x: 5 }; if x.a()!=5{return 1;} if x.b()!=10{return 2;} return 0; }