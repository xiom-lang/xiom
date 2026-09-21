// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-R08: missing closing parenthesis -- parenthesized expression depth stress
fn main() -> Int { var x = (((((1 + 2) + 3) + 4) + 5) + 6); return 0; }
