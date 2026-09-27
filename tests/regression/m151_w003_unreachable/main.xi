// m151 (Stage 6 W003): statements after an unconditional diverger warn
// (return, `while true` without a break, and an if whose every path
// diverges). Compilation stays exit 0.
module m151_w003_unreachable;

use xiom.io;

fn dead_after_return() -> Int {
  return 1;
  io.println("dead-return");
  2
}

fn dead_after_loop() -> Int {
  while true { }
  io.println("dead-loop");
  return 0;
}

fn dead_after_if() -> Int {
  if true { return 1; } else { return 2; }
  io.println("dead-if");
  return 0;
}

fn main() -> Int {
  io.println(dead_after_return().to_str());
  return 0;
}
