// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m76: contract payload clauses mixing `result.value`/`result.error` with a
// PARAM receiver. Pre-fix, the implication's bare `is Some/Err` rebind bound
// the scrutinee to the payload slot, but a `.value`/`.error` field read on
// that name fell through the emitter's literal-0 fallback, so
// `result.value.len()` evaluated `xiom_str_len(NULL)` (-1) and the clause
// aborted with a spurious "contract violated: ensures" for a value that
// satisfies it. The rebind now maps the payload-field read to the payload.
module m76_contract_payload_param_len

use xiom.io;

// Payload len vs a constant bound (always passed).
fn wa(s: Str) -> Option[Str]
  ensures: result is Some => result.value.len() >= 0
{
  return Some(s);
}

// Payload len vs a PARAM len -- the R18 false positive.
fn wb(s: Str) -> Option[Str]
  ensures: result is Some => result.value.len() <= s.len()
{
  return Some(s);
}

// Payload VALUE vs the param (locks the actual payload handle, not just len).
fn wc(s: Str) -> Option[Str]
  ensures: result is Some => result.value == s
{
  return Some(s);
}

// Scalar payload vs a param-derived bound.
fn wd(n: Int) -> Option[Int]
  ensures: result is Some => result.value <= n + 1
{
  return Some(n);
}

// Err-side payload read (`result.error`) on a bare `is Err` rebind.
fn we(s: Str) -> Result[Int, Str]
  ensures: result is Err => result.error.len() <= s.len()
{
  return Err(s);
}

fn main() -> Int {
  match wa("abc") { Some(v) => { if v.len() != 3 { return 1; } }, None => { return 2; } }
  match wb("abc") { Some(v) => { if v.len() != 3 { return 3; } }, None => { return 4; } }
  match wc("abc") { Some(v) => { if v != "abc" { return 5; } }, None => { return 6; } }
  match wd(7) { Some(v) => { if v != 7 { return 7; } }, None => { return 8; } }
  match we("bad") { Ok(v) => { return 9; }, Err(e) => { if e != "bad" { return 10; } } }
  io.println("M76 OK");
  return 0;
}
