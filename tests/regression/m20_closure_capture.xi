// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 10; var f = |y| x + y; var r = f(5); if r != 15 { return 1; } return 0; }