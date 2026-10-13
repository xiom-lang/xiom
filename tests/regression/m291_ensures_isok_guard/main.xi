// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m291 lock (REL-20261010-1548-stdlib-7): the GUARD form
// `(result.is_ok == true) => (result.len() == s.len())` must rebind `result`
// to the Ok payload like the canonical `result is Ok => ...` form -- the
// consequence's `result.len()` used to compile against the unbound Result
// (a stubbed @Result.len returning 0) and the clause aborted at runtime.
// The m283 contract guard had also been skipping every clause check
// (`d > 0` with d == 1 at the first clause site), which masked this; both
// are fixed in m291 and this lock runs with enforcement LIVE.
module m291_ensures_isok_guard.main

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

fn main() -> Int {
  let a = canonical_len("ab");
  if !a.is_ok { return 2; }
  let b = guarded_len("ab");
  if !b.is_ok { return 3; }
  return 0;
}
