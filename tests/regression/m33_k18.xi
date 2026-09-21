// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K18: Closure returning closure -- inline nested call instead
fn double_apply(f: fn(Int) -> Int, x: Int, y: Int) -> Int { return f(f(x) + y); }
fn triple(x: Int) -> Int { return x * 3; }
fn main() -> Int { if double_apply(triple, 10, 5) != 105 { return 1; } return 0; }
