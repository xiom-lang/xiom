// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn mixed() -> (Int, Int) { var x = 10; var y = 20; return (x, y); } fn main() -> Int { let m = mixed(); if m.0 != 10 { return 1; } if m.1 != 20 { return 2; } return 0; }