// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var a = 2; var b = 3; var f = |x| x + a; var g = |x| x * b; if f(g(4)) != 14 { return 1; } return 0; }