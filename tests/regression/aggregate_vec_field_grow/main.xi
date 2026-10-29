// REL-20261028-1641: an aggregate binding's Vec field can be REPLACED after
// the binding (push growth through `s.buf.push`, or growth inside a `&mut`
// callee). The scope-exit drop must read the field at exit, not a pointer
// captured at binding time -- the captured pointer was freed by realloc and
// freed again at exit (0xC0000374 / ASan double-free in p_wave162).
module aggregate_vec_field_grow

use xiom.io;
use xiom.convert;

type S = { buf: Vec[UInt8]; marks: Vec[Int]; n: Int; }

fn grow_in_place(s: &mut S, count: Int) -> Int {
  var i = 0;
  while i < count {
    s.buf.push(7);
    s.marks.push(i);
    i = i + 1;
  }
  return s.buf.len();
}

fn main() -> Int {
  // 1. Growth through the local's own field (no call involved).
  var a = S{ buf: Vec[UInt8].new(), marks: Vec[Int].new(), n: 1 };
  var i = 0;
  while i < 4096 {
    a.buf.push(1);
    i = i + 1;
  }
  if a.buf.len() != 4096 { return 1; }
  if a.buf[4095] != 1 { return 2; }

  // 2. Growth through a &mut callee (the p_wave162 shape).
  var b = S{ buf: Vec[UInt8].new(), marks: Vec[Int].new(), n: 2 };
  let len = grow_in_place(&mut b, 4096);
  if len != 4096 { return 3; }
  if b.buf.len() != 4096 { return 4; }
  if b.marks.len() != 4096 { return 5; }
  if b.buf[0] != 7 { return 6; }
  if b.marks[4095] != 4095 { return 7; }
  if b.n != 2 { return 8; }

  // 3. A field that never grows still drops normally.
  var c = S{ buf: Vec[UInt8].new(), marks: Vec[Int].new(), n: 3 };
  c.buf.push(9);
  if c.buf.len() != 1 { return 9; }

  io.println("aggregate vec field grow ok");
  return 0;
}
