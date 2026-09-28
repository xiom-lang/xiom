// m153 (C001, benchmark relay): `io.parse_int`'s body does
// `let trimmed = s.trim(); if trimmed.is_empty() { ... }`. `trim` is a
// receiver-sugar FREE fn, so the let binding's XIOM type was never recorded;
// the later `trimmed.is_empty()` then mis-resolved the receiver as a MODULE,
// collapsed to the bare `is_empty` alias and emitted an undefined
// `@is_empty` (C001: unresolved symbol from @io.parse_int).
//
// The fix synthesises the primitive method key from the receiver local's
// LLVM type when the XIOM type is missing (self-validating: only when the
// `Str.is_empty` key exists). This lock exercises both the exact shape and
// the runtime result.
module m153_parse_int_trim_isempty;

use xiom.io;

fn parse_checked(s: Str) -> Result[Int, Str] {
  let trimmed = s.trim();
  if trimmed.is_empty() {
    return Err("empty input");
  }
  match io.parse_int(trimmed) {
    Ok(v) => { return Ok(v); },
    Err(_) => { return Err("bad digits"); },
  }
}

fn main() -> Int {
  match parse_checked(" 42 ") {
    Ok(v) => { if v != 42 { return 1; } },
    Err(_) => { return 2; },
  }
  match parse_checked("   ") {
    Ok(_) => { return 3; },
    Err(e) => { if e != "empty input" { return 4; } },
  }
  match io.parse_int("7") {
    Ok(v2) => { if v2 != 7 { return 5; } },
    Err(_) => { return 6; },
  }
  return 0;
}
