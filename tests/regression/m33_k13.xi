// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K13: Closure type annotation -- explicit block closure with typed params
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int { var result = apply(square, 7); if result != 49 { return 1; } return 0; }
