// BUG 56 regression: parser dropped the fn BODY when an ensures clause
// preceded an EXPRESSION body -- `ensures: result == x` + `{ x }` parsed
// `x { x }` as a STRUCT LITERAL (consuming the body block) -> the fn
// emitted ret 0 / lost its param (smoke_stress_convert_identity family).
module m37_bug56_ensure_expr_body
use xiom.convert;

fn main() -> Int {
  var v = convert.identity(42);
  if v != 42 { return 1; }
  return 0;
}
