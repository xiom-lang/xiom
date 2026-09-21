// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K17: Closure with compound assignment operations
fn main() -> Int { var a = 10; var b = 20; a += 5; b -= 5; var f = |x| x + a + b; if f(5) != 35 { return 1; } return 0; }
