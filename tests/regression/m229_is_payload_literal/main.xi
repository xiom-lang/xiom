// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m229 (packages lane): `expr is Ok(<literal>)` ignored the payload literal
// and compared only the tag -- `pick(2) is Ok(1)` was true. Literal payloads
// in `is` patterns must be compared after the tag check; binding/wildcard
// forms keep their semantics.
module m229_is_payload_literal

fn pick(n: Int) -> Result[Int, Int] {
  return Ok(n);
}

fn maybe(n: Int) -> Option[Int] {
  if n < 0 { return None; }
  return Some(n);
}

fn main() -> Int {
  // Int literal payloads (Result).
  if pick(2) is Ok(1) { return 1; }
  if !(pick(1) is Ok(1)) { return 2; }
  if !(pick(2) is Ok(2)) { return 3; }

  // Int literal payloads (Option) including the None case.
  if maybe(2) is Some(1) { return 4; }
  if !(maybe(1) is Some(1)) { return 5; }
  if maybe(-1) is Some(1) { return 6; }

  // Binding form still works and reads the right payload.
  let r = pick(9);
  match r {
    Err(_) => { return 7; }
    Ok(v) => { if v != 9 { return 8; } }
  }
  return 0;
}
