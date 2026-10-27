// REL-20261027-1717-engine: local struct literals owning fresh Vec fields
// must drop those fields at the caller's scope, including when the struct is
// borrowed (consume_ref) or moved by value to a non-retaining consumer.
module aggregate_vec_leak

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

type Big = {
  data: Vec[UInt8];
  tag: Int;
}

fn consume_ref(b: &Big) -> Int {
  return b.data.len();
}

fn consume_val(b: Big) -> Int {
  return b.data.len();
}

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var v = Vec[UInt8].with_len(sz);
    let b = Big{ data: v; tag: 2 };
    sink = sink + consume_ref(&b);
    let b2 = Big{ data: Vec[UInt8].with_len(sz); tag: 3 };
    sink = sink + consume_val(b2);
    i = i + 1;
  }
  io.println("aggregate vec leak done sink=" + convert.int_to_string(sink));
  return 0;
}
