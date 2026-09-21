// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var sum = 0; var i = 0; while i < 10 { var j = 0; while j < 10 { sum = sum + 1; j = j + 1; } i = i + 1; } if sum != 100 { return 1; } return 0; }