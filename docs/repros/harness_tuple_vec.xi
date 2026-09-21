// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_tuple_vec;

fn main() -> Int {
  var a = Vec[UInt8].new();
  a.push(1); a.push(2); a.push(3);
  var b = Vec[UInt8].new();
  b.push(9); b.push(8);
  var r = gcm_like(a, b);
  if !r.is_ok() { return 1; }
  var t = r.unwrap();
  var x = t._0;
  var y = t._1;
  if x.len() != 3 { return 2; }
  if y.len() != 2 { return 3; }
  if x[2] != 3 { return 4; }
  if y[1] != 8 { return 5; }
  return 0;
}
