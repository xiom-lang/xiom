// Relay lock gateway: both private Timers arrive as catalog modules through
// the package graph; each must keep its own qualified `%struct` layout.
module m134.main

use m134.alpha as al;
use m134.beta as be;

fn main() -> Int {
  let a = al.make();
  let b = be.use_it();
  if a != 1 { return 1; }
  if b != 2 { return 2; }
  return 0;
}
