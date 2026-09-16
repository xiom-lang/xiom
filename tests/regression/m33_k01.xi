// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K01: Pipe closure -- simple identity function
fn main() -> Int { var id = |x| x; if id(42) != 42 { return 1; } return 0; }
