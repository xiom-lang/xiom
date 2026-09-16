// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn maybe_val(o: Option[Int]) -> Int { match o { Some(v) => { return v; } None => { return 0; } } }
fn main() -> Int { if maybe_val(Some(7)) != 7 { return 1; } if maybe_val(None) != 0 { return 2; } return 0; }