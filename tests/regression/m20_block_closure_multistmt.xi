// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 5; var f = (fn(y: Int) -> Int { var t = x + y; return t * 2; }); if f(3) != 16 { return 1; } return 0; }