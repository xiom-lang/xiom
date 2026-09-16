// XIOM -- phase1_ownership
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

fn take_ownership(x: Int) -> Int { return x + 1; }
fn read_borrow(x: &Int) -> Int { return x + 0; }
fn main() -> Int { let a = take_ownership(41); let b = read_borrow(&a); return a + b; }
