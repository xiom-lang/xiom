// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

interface Val { fn val(self) -> Int; }
type N = { n: Int; }
impl Val for N { fn val(self) -> Int { return self.n; } }
fn main() -> Int { var n = N{ n: 77 }; if n.val()!=77{return 1;} return 0; }