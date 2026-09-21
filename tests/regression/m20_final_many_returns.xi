// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn abs(x: Int) -> Int { if x < 0 { return -x; } return x; }
fn main() -> Int { if abs(-5)!=5{return 1;} if abs(5)!=5{return 2;} if abs(0)!=0{return 3;} return 0; }