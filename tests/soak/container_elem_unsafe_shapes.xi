// Element-drop ownership tag NEGATIVE control: every shape below must keep
// the historical leak (and, critically, must NOT free a buffer that another
// binding still owns). A crash/double free here means the tag is unsound.
module container_elem_unsafe_shapes

use xiom.io;

type R = {
  id: Int;
  xs: Vec[UInt8];
}

fn touch(v: &Vec[R]) -> Int {
  return v.len();
}

fn main() -> Int {
  // A: duplicate push of one aggregate (E001 warns; the header aliases).
  var a = Vec[R].new();
  var xa = Vec[UInt8].with_len(1024);
  var ra = R{ id: 1; xs: xa };
  a.push(ra);
  a.push(ra);
  if a.len() != 2 { return 1; }

  // B: pop transfers an element out.
  var b = Vec[R].new();
  var xb = Vec[UInt8].with_len(1024);
  var rb = R{ id: 2; xs: xb };
  b.push(rb);
  match b.pop() {
    Some(o) => { if o.id != 2 { return 2; } },
    None => { return 2; },
  }

  // C: an element's Vec field escapes into a binding.
  var c = Vec[R].new();
  var xc = Vec[UInt8].with_len(1024);
  var rc = R{ id: 3; xs: xc };
  c.push(rc);
  let escaped = c[0].xs;
  if escaped.len() != 1024 { return 3; }

  // D: with_len container (uninitialized elements).
  var d = Vec[R].with_len(4);
  if d.len() != 4 { return 4; }

  // E: borrow of the container to a call.
  var e = Vec[R].new();
  var xe = Vec[UInt8].with_len(1024);
  var re = R{ id: 5; xs: xe };
  e.push(re);
  if touch(&e) != 1 { return 5; }

  io.println("container elem unsafe shapes ok");
  return 0;
}
