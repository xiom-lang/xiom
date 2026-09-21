// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 1; var b = a << 4; if b != 16 { return 1; } if b >> 2 != 4 { return 2; } return 0; }