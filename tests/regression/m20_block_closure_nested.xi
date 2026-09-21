// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 100; var f = (fn(x: Int) -> Int { return a + x; }); if f(23) != 123 { return 1; } return 0; }