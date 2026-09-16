// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var p: *Int; unsafe { p = 0 as *Int; if p != (0 as *Int) { return 1; } } return 0; }