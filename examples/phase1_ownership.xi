// XIOM -- phase1_ownership
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn take_ownership(x: Int) -> Int { return x + 1; }
fn read_borrow(x: &Int) -> Int { return x + 0; }
fn main() -> Int { let a = take_ownership(41); let b = read_borrow(&a); return a + b; }
