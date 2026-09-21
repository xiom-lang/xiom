// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var f = (fn(x: Int, y: Int) -> Int { return x * y; }); if f(6, 7) != 42 { return 1; } return 0; }