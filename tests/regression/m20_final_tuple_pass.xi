// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn swap(a:Int,b:Int) -> (Int,Int) { return (b,a); }
fn main() -> Int { var p = swap(1,2); if p.0!=2{return 1;} if p.1!=1{return 2;} return 0; }