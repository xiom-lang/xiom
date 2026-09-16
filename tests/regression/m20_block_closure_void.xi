// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var x = 1; var f = (fn() -> Int { return x; }); if f() != 1 { return 1; } return 0; }