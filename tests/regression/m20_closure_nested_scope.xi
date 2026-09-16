// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var outer = 100; var f = |x| outer + x; if f(5) != 105 { return 1; } return 0; }