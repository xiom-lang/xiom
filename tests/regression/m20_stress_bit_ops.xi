// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 0xFF; var b = 0x0F; if (a & b) != 0x0F { return 1; } if (a | b) != 0xFF { return 2; } return 0; }