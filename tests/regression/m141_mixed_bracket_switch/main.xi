// Packages/stdlib relay lock: mixed generic brackets (`Result[Str, Str>`)
// are accepted by the LAX transition default and rejected when
// XIOM_STRICT_BRACKETS=1 is set. Asserted by checker_locks.rs; the default
// flip is HELD for v0.62.0 (the pin carries 3 stdlib mixed sites) and lands
// with the next release once the stdlib wave canonicalizes them.
module m141_mixed_bracket_switch;

fn g() -> Result[Str, Str> {
  return Ok("x");
}

fn main() -> Int {
  let r = g();
  if !r.is_ok() { return 1; }
  return 0;
}
