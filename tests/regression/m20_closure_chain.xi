// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn make_adder(n: Int) -> Int { var f = |x| n + x; return f(5); } fn main() -> Int { if make_adder(10) != 15 { return 1; } return 0; }