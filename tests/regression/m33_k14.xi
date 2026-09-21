// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K14: Closure with early return inside conditional
fn main() -> Int { var x = 5; var f = |y| x + y; if f(0) > 100 { return 1; } if f(10) != 15 { return 2; } return 0; }
