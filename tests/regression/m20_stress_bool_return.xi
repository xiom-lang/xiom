// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn is_pos(n: Int) -> Bool { return n > 0; }
fn main() -> Int { if !is_pos(5) { return 1; } if is_pos(0) { return 2; } if is_pos(-1) { return 3; } return 0; }