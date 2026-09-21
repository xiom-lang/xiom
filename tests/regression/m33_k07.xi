// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K07: Direct call -- function passed as parameter (function pointer type)
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int { if apply(square, 5) != 25 { return 1; } return 0; }
