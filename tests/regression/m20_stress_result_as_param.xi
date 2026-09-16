// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn check_val(r: Result[Int, Bool]) -> Int { match r { Ok(v) => { return v; } Err(_) => { return -1; } } }
fn main() -> Int { if check_val(Ok(42)) != 42 { return 1; } if check_val(Err(false)) != -1 { return 2; } return 0; }