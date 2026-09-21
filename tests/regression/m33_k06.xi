// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K06: Block closure -- void (no params), captures local
fn main() -> Int { var x = 1; var f = fn() -> Int { return x; }; if f() != 1 { return 1; } return 0; }
