// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

type D = { a: Int; b: Int; }
fn main() -> Int { var d = D{ a: 7, b: 8 }; if d.a+d.b != 15 { return 1; } return 0; }