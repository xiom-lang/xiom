// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_opt_vec;

fn main() -> Int {
  var o = captures("hello", "x");
  if !o.is_some() { return 1; }
  var v = o.unwrap();
  if v.len() != 1 { return 2; }
  if v[0] != "hello" { return 3; }
  return 0;
}
