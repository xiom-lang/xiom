// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum R { Ok(val: Int), Err(msg: Str) }
fn div(a:Int,b:Int) -> R { if b==0{return R.Err("no");} return R.Ok(a/b); }
fn main() -> Int { match div(10,2){R.Ok(v)=>{if v!=5{return 1;}} R.Err(_)=>{return 2;}} match div(10,0){R.Ok(_)=>{return 3;} R.Err(_)=>{} } return 0; }