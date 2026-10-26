// Control: the same allocation with the unsafe block INSIDE the loop, so
// the arena is entered/exited (and its slabs freed) every iteration.
module arena_scope_loop_control

use xiom.io;
use xiom.convert;

fn leak(n: Int) -> Int
  requires: n >= 0
{
  var i = 0;
  var sink = 0;
  while i < n {
    unsafe {
      let v = Vec[Int].with_capacity(65536);
      v.push(1);
      sink = sink + v.len();
    }
    i = i + 1;
  }
  return sink;
}

fn main() -> Int {
  let sink = leak(4000);
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
