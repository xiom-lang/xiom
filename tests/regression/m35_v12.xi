// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V12: Vec[Int16] -- small integer Vec stress
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int16].new();
  v.push(100 as Int16);
  v.push(200 as Int16);
  v.push(300 as Int16);
  if v.len() != 3 { return 1; }
  if v[0] != 100 as Int16 { return 2; }
  if v[1] != 200 as Int16 { return 3; }
  if v[2] != 300 as Int16 { return 4; }
  var p = v.pop();
  var expected: Option[Int16] = Some(300 as Int16);
  if p != expected { return 5; }
  // Remove first
  v.remove(0);
  if v.len() != 1 { return 6; }
  if v[0] != 200 as Int16 { return 7; }
  return 0;
}
