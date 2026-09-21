// XIOM -- phase1_interface
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

interface Comparable { fn compare(other: &Int) -> Int; }
fn is_greater(a: Int, b: Int) -> Bool { return a > b; }
fn main() -> Int { if is_greater(10, 5) { return 1; } return 0; }
