// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 10; if x > 5 { x = 20; } if x != 20 { return 1; } if x < 0 { x = -1; } if x != 20 { return 2; } return 0; }