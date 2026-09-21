// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 0; unsafe { x = 42; } if x != 42 { return 1; } return 0; }