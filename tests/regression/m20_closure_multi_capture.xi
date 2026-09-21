// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 10; var b = 20; var f = |x| a + b + x; if f(0) != 30 { return 1; } return 0; }