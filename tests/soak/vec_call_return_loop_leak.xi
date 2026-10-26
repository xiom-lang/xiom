// Engine witness (2026-10-25): a function returns a fresh Vec; the caller
// binds and drops it in a loop. v0.65.0 treats call returns as aliases and
// never drops them, so a fresh Vec leaks its full size per call
// (font_shape / ui2_draw_text / io.read_at class).
module vec_call_return_loop_leak

use xiom.io;
use xiom.convert;

fn make(n: Int) -> Vec[Int]
  requires: n >= 0
{
  return Vec[Int].with_len(n);
}

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 400000 {
    let v = make(64);
    sink = sink + v.len();
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
