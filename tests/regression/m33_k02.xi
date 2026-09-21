// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K02: Pipe closure -- capturing local variable
fn main() -> Int { var base = 100; var f = |x| base + x; if f(23) != 123 { return 1; } return 0; }
