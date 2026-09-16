// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn triple() -> (Int, Int, Int) { return (10, 20, 30); } fn main() -> Int { let t = triple(); if t.0 != 10 { return 1; } if t.1 != 20 { return 2; } if t.2 != 30 { return 3; } return 0; }