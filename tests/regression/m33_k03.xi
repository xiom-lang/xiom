// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K03: Pipe closure -- multiple parameters
fn main() -> Int { var add = |x, y| x + y; if add(7, 8) != 15 { return 1; } return 0; }
