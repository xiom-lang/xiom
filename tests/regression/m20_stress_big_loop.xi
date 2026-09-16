// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var sum = 0; var i = 0; while i < 1000 { sum = sum + i; i = i + 1; } if sum != 499500 { return 1; } return 0; }