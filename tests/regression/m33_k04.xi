// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K04: Block closure -- typed params, direct assignment (no parens)
fn main() -> Int { var dbl = fn(x: Int) -> Int { return x * 2; }; if dbl(21) != 42 { return 1; } return 0; }
