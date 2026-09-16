// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum E { A, B(val: Int), C(x: Int, y: Int) }
fn f(e: E) -> Int { match e { A => { return 1; } B(v) => { return v; } C(x, y) => { return x + y; } } }
fn main() -> Int { if f(E.A) != 1 { return 1; } if f(E.B(10)) != 10 { return 2; } if f(E.C(3, 7)) != 10 { return 3; } return 0; }