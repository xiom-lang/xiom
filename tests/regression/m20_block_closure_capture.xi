// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 10; var f = (fn(y: Int) -> Int { return x + y; }); if f(5) != 15 { return 1; } return 0; }