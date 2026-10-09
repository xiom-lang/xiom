// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m247 (stdlib ensures-isok finding): method-form variant guards in
// ensures implications must carry the same payload knowledge as the
// canonical `result is Ok =>` form -- `(result.is_ok == true) =>` and
// the bare `result.is_ok =>` spellings used to call the erased
// Result.len and false-violate at runtime.
module m247_isok_guard_imply

fn canonical_len(s: Str) -> Result[Str, Str]
  ensures: result is Ok => result.len() == s.len()
{
  return Ok(s);
}

fn guarded_len(s: Str) -> Result[Str, Str]
  ensures: (result.is_ok == true) => (result.len() == s.len())
{
  return Ok(s);
}

fn bare_len(s: Str) -> Result[Str, Str]
  ensures: result.is_ok => result.len() == s.len()
{
  return Ok(s);
}

// Err return: the guarded implication is vacuously true and must not
// execute any payload read.
fn err_ok() -> Result[Str, Str]
  ensures: (result.is_ok == true) => (result.len() == 3)
{
  return Err("nope");
}

// Boxed struct payload (Vec): the consequence must unbox and dispatch
// Vec.len on the payload, not the erased Result.
fn vec_len() -> Result[Vec[Int], Str]
  ensures: (result.is_ok == true) => (result.len() == 2)
{
  var v = Vec[Int].new();
  v.push(7); v.push(8);
  return Ok(v);
}

fn main() -> Int {
  let a = canonical_len("ab");
  if !a.is_ok { return 2; }
  let b = guarded_len("ab");
  if !b.is_ok { return 3; }
  let c = bare_len("ab");
  if !c.is_ok { return 4; }
  let d = err_ok();
  if d.is_ok { return 5; }
  let e = vec_len();
  if !e.is_ok { return 6; }
  return 0;
}
