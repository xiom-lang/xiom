// REL-20261027-2034-engine: a callee that moves a Vec field out of a
// by-value aggregate parameter must own and drop the moved buffer. 1000 x
// 512 KB; the v0.65.2 pass leaks ~0.37 MB/iter (the moved local is not a
// drop candidate because it is bound from a param field, and the caller does
// not drop the aggregate it passed by value).
module byval_param_field_move_leak

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

type Big = {
  data: Vec[UInt8];
  tag: Int;
}

fn move_big_field(b: Big) -> Int {
  var v = b.data;
  return v.len();
}

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var v = Vec[UInt8].with_len(sz);
    let b = Big{ data: v; tag: 2 };
    sink = sink + move_big_field(b);
    i = i + 1;
  }
  io.println("byval param field move done sink=" + convert.int_to_string(sink));
  return 0;
}
