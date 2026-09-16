// XIOM -- phase1_interface
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

interface Comparable { fn compare(other: &Int) -> Int; }
fn is_greater(a: Int, b: Int) -> Bool { return a > b; }
fn main() -> Int { if is_greater(10, 5) { return 1; } return 0; }
