// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

var counter: Int;
fn inc() -> Int { counter = counter + 1; return counter; }
fn main() -> Int { counter = 0; if inc() != 1 { return 1; } if inc() != 2 { return 2; } if counter != 2 { return 3; } return 0; }