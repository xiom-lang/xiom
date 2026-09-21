// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int { var o = Some(5); match o { Some(v) => { if v!=5{return 1;} } None => { return 2; } } return 0; }