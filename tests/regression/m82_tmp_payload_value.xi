// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m82: R28 lock -- Option/Result `.value` on a TEMPORARY call result must
// materialize the aggregate payload (Vec), not bind the raw payload handle
// as i64. Pre-fix: `opt_vec().value` produced a Vec whose element reads
// emitted a literal 0 (zeroed data); a named local worked.
module m82_tmp_payload_value

fn opt_vec() -> Option[Vec[Int]] {
  var v = Vec.new[Int]();
  v.push(4);
  v.push(6);
  return Some(v);
}

fn res_vec() -> Result[Vec[Int], Str] {
  var v = Vec.new[Int]();
  v.push(9);
  return Ok(v);
}

fn opt_int() -> Option[Int] { return Some(41); }

fn main() -> Int {
  // R28: temporary Option[Vec].value
  let a = opt_vec().value;
  if a.len() != 2 { return 1; }
  if a[0] != 4 { return 2; }
  if a[1] != 6 { return 3; }

  // R28: temporary Result[Vec, Str].value
  let b = res_vec().value;
  if b.len() != 1 { return 4; }
  if b[0] != 9 { return 5; }

  // scalar payload temporary (control: was already correct)
  let c = opt_int().value;
  if c != 41 { return 6; }

  // named-local control
  var o = opt_vec();
  let d = o.value;
  if d[0] != 4 { return 7; }

  return 0;
}
