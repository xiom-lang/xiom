// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N15: Derive on type with optional-like fields -- struct with Eq using Int fields only
type Config = { port: Int; host_id: Int; } derive[Eq]
fn main() -> Int {
  var a = Config{ port: 8080; host_id: 1; };
  var b = Config{ port: 8080; host_id: 1; };
  var c = Config{ port: 3000; host_id: 2; };
  if a == b && a != c { return 0; }
  return 1;
}
