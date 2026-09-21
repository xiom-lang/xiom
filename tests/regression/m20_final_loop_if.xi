// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var i = 0; var s = 0; while i < 10 { if i % 2 == 0 { s = s + i; } i = i + 1; } if s != 20 { return 1; } return 0; }