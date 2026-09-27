// m151 (Stage 6 W003) negative: a return behind an if branch the block
// continues from, a loop with a reachable break, and `debugger` must all
// stay silent.
module m151_w003_guarded;

use xiom.io;

fn guarded(n: Int) -> Int {
  if n < 0 {
    return -1;
  }
  var i = 0;
  while i < n {
    if i == 3 { break; }
    i = i + 1;
  }
  debugger;
  io.println(i.to_str());
  return i;
}

fn main() -> Int {
  if guarded(5) != 3 { return 1; }
  return 0;
}
