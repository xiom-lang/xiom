// m394 (Stage 6 tier 3 W013) negative: distinct places and computed updates
// must stay silent under --lint-strict.
module m394_w013_guard;

fn main() -> Int {
  var x = 1;
  var y = 2;
  x = y;
  x = x + 1;
  y = x;
  if x != 2 || y != 2 { return 1; }
  return 0;
}
