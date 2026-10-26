// Engine witness (2026-10-25): Vec[Int].with_len(64) allocated and dropped
// in a loop. v0.65.0 grows ~512 B/vec (the full allocation total), i.e. the
// buffer is never freed. 400k iterations must stay bounded.
module vec_with_len_loop_leak

use xiom.io;
use xiom.convert;

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 400000 {
    let v = Vec[Int].with_len(64);
    sink = sink + v.len();
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
