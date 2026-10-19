// Editor-shaped escaped-Vec witness (REL-20261019-1343-engine):
// a function returns a frame-like struct containing several Vecs; the caller
// consumes one frame per loop iteration. m368 only reclaims non-escaping Vec
// locals, so the frame Vecs are expected to leak until ownership/drop reaches
// struct fields or a scoped arena is used.
module escaped_frame_witness

use xiom.io;
use xiom.convert;

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
    let f = make_frame(i);
    sink = sink + consume(f);
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
