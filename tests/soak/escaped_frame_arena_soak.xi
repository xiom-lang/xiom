// Large scoped-arena witness for REL-20261019-1343-engine.
// Same frame shape as escaped_frame_vec_leak.xi, but each frame is wrapped in
// the runtime guard arena; all Vec data allocated by make_frame/consume is
// reclaimed wholesale at exit. Expected: flat WS, correct sink.
module escaped_frame_arena_soak

use xiom.io;
use xiom.convert;

extern "C" {
  fn xiom_guard_heap_enter();
  fn xiom_guard_heap_exit();
}

type Frame = { a: Vec[Int]; b: Vec[Float64]; c: Vec[Int]; }

fn make_frame(n: Int) -> Frame {
  var a = Vec[Int].new();
  var j = 0;
  while j < 1000000 { a.push(n); j = j + 1; }
  var b = Vec[Float64].new();
  j = 0;
  while j < 1000000 { b.push(1.0); j = j + 1; }
  var c = Vec[Int].new();
  j = 0;
  while j < 1000000 { c.push(2); j = j + 1; }
  return Frame{ a: a; b: b; c: c };
}

fn consume(f: Frame) -> Int {
  return f.a.len() + f.b.len() + f.c.len();
}

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 200 {
    xiom_guard_heap_enter();
    let f = make_frame(i);
    sink = sink + consume(f);
    xiom_guard_heap_exit();
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
