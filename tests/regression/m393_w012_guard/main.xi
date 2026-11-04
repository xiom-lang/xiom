// m393 (Stage 6 tier 3 W012) negative: variable conditions and the
// intentional `while true` server-loop shape must stay silent.
module m393_w012_guard;

fn main() -> Int {
  let c = true;
  var n = 0;
  if c { n = 1; }
  while n < 3 { n = n + 1; }
  while true {
    n = n + 1;
    if n > 5 { break; }
  }
  if n != 6 { return 1; }
  return 0;
}
