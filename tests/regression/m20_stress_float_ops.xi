// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 1.5; var b = 2.5; var c = a + b; if c != 4.0 { return 1; } var d = b - a; if d != 1.0 { return 2; } var e = a * 2.0; if e != 3.0 { return 3; } var f = 10.0 / 4.0; if f != 2.5 { return 4; } return 0; }