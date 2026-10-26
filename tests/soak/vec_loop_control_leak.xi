// m375 liveness witness: loop-local Vecs must also be reclaimed on the
// `continue` and `break` paths, not only at the fall-through back-edge.
module vec_loop_control_leak

use xiom.io;
use xiom.convert;

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 400000 {
    let v = Vec[Int].with_len(64);
    sink = sink + v.len();
    i = i + 1;
    if i % 7 == 0 {
      continue;
    }
    if i == 399999 {
      break;
    }
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
