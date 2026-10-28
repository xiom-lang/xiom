// Element-drop ownership tag witness: a push-only `Vec[R]` where every
// pushed aggregate is a locally built record owning its Vec field. The
// container drop must free each element's Vec payload (pre-fix this leaked
// 496 MB per 1000 iterations of 512 KB).
module container_elem_owned_drop

use xiom.io;
use xiom.convert;

type R = {
  id: Int;
  xs: Vec[UInt8];
}

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var rs = Vec[R].new();
    var xs = Vec[UInt8].with_len(sz);
    var r = R{ id: i; xs: xs };
    rs.push(r);
    // A SCALAR element read must stay allowed (it cannot alias the buffer).
    sink = sink + rs[0].id;
    i = i + 1;
  }
  io.println("container elem owned done sink=" + convert.int_to_string(sink));
  return 0;
}
