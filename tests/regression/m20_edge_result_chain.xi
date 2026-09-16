// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn div(a: Int, b: Int) -> Result[Int, Str] { if b == 0 { return Err("div0"); } return Ok(a / b); }
fn main() -> Int { match div(10, 2) { Ok(v) => { if v != 5 { return 1; } } Err(_) => { return 2; } } match div(10, 0) { Ok(_) => { return 3; } Err(_) => {} } return 0; }