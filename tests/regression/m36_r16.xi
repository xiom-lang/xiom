// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-R16: reserved word misuse -- contextual keyword test, function with match
fn main() -> Int { var x = 0; match x { 0 => { return 0; } _ => { return 1; } } }
