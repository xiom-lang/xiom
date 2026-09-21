// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O14: Option is_none -- direct .is_none() method call
fn main() -> Int {
  var a = Some(42);
  var b: Option[Int] = None;
  var c = Some(true);
  if a.is_none() != false { return 1; }
  if b.is_none() != true { return 2; }
  if c.is_none() != false { return 3; }
  var d: Option[Float64] = None;
  if d.is_none() != true { return 4; }
  var e = Some(0.0);
  if e.is_none() != false { return 5; }
  return 0;
}
