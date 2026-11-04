// m394 (Stage 6 tier 3 W013): `x = x` self-assignment warns only under
// --lint-strict. Warning-only; the compile exits 0.
module m394_w013_self_assign;

fn main() -> Int {
  var x = 1;
  var y = 2;
  x = x;
  y = y;
  if x != 1 || y != 2 { return 1; }
  return 0;
}
