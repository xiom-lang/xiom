// Arena-scope leak witness: a loop INSIDE one unsafe block allocates a
// large Vec every iteration. The guard arena is active for the whole block,
// so each iteration's allocation stays in the arena until block exit.
// Expected defect: RSS grows ~alloc x iterations. Expected fix: bounded.
module arena_scope_loop_leak

use xiom.io;
use xiom.convert;

fn leak(n: Int) -> Int
  requires: n >= 0
{
  var i = 0;
  var sink = 0;
  unsafe {
    while i < n {
      let v = Vec[Int].with_capacity(65536);
      v.push(1);
      sink = sink + v.len();
      i = i + 1;
    }
  }
  return sink;
}

fn main() -> Int {
  let sink = leak(4000);
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
