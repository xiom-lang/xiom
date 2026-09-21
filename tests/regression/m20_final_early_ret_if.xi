// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn f(x: Int) -> Int { if x < 0 { return 0; } return x; }
fn main() -> Int { if f(-1)!=0{return 1;} if f(5)!=5{return 2;} return 0; }