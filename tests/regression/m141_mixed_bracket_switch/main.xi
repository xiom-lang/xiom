// Packages/stdlib relay lock: mixed generic brackets (`Result[Str, Str>`)
// are accepted by the LAX transition default and rejected when
// XIOM_STRICT_BRACKETS=1 is set. Asserted by checker_locks.rs; the default
// flips at the pin bump once the stdlib wave lands.
module m141_mixed_bracket_switch;

fn g() -> Result[Str, Str> {
  return Ok("x");
}

fn main() -> Int {
  let r = g();
  if !r.is_ok() { return 1; }
  return 0;
}
