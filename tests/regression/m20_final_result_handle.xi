// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var r = Ok(42); match r { Ok(v) => { if v!=42{return 1;} } Err(_) => { return 2; } } return 0; }