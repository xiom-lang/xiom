// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m217 (C-PULSE-10): nested payload chains must keep the INNER payload type.
// `Result[Option[Str], Str]` read as `c.value.value` used to keep the erased
// i64 slot for the inner `.value` (field_payload_xiom had no Field arm), so
// the Str data pointer was interpreted as an Int -- the xiom.kv consumer saw
// `kv_get` print an address-like decimal while `kv_get_bytes` was correct.
// Both construction paths are locked: direct `Ok(Some(...))` and the
// match-mediated form.

module repro_nested_payload

fn mk_bytes() -> Vec[UInt8] {
  var out: Vec[UInt8] = Vec[UInt8].new();
  var i = 0;
  while i < 10 {
    out.push((97 + i) as UInt8);
    i = i + 1;
  }
  return out;
}

fn get_nested() -> Result[Option[Str], Str] {
  let b = mk_bytes();
  return Ok(Some(Str::from_utf8(b)));
}

fn get_nested_match() -> Result[Option[Str], Str] {
  let b = mk_bytes();
  var o: Option[Str] = Some(Str::from_utf8(b));
  match o {
    Some(s) => { return Ok(Some(s)); },
    None => {},
  }
  return Ok(None);
}

fn main() -> Int {
  let c = get_nested();
  if c.value.value != "abcdefghij" { return 1; }
  let d = get_nested_match();
  if d.value.value != "abcdefghij" { return 2; }
  return 0;
}
