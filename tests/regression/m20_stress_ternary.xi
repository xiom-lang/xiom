// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn ternary(a:Bool, b:Int, c:Int) -> Int { if a { return b; } return c; }
fn main() -> Int { if ternary(true, 10, 20) != 10 { return 1; } if ternary(false, 10, 20) != 20 { return 2; } return 0; }