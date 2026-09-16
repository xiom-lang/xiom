// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

type B = { n: Int; }
fn B.double(self) -> Int { return self.n * 2; }
fn main() -> Int { var b = B{ n: 21 }; if b.double() != 42 { return 1; } return 0; }