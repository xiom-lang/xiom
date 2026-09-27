// m151 (Stage 6 W002) negative: guard-first recursion (recursive call after
// a conditional return) and branch-reached mutual recursion must stay
// silent -- a guard before the recursive call means the cycle is not
// unconditional.
module m151_w002_guard;

use xiom.io;

fn fact(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * fact(n - 1);
}

fn ping(n: Int) -> Int {
  if n <= 0 { return 0; }
  return pong(n - 1);
}

fn pong(n: Int) -> Int {
  if n <= 0 { return 0; }
  return ping(n - 1);
}

fn main() -> Int {
  if fact(5) != 120 { return 1; }
  if ping(4) != 0 { return 2; }
  return 0;
}
