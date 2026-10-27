// REL-20261027-1717-engine: the call-returned aggregate form. The constructor
// is annotated #[owns_return]; the caller drops the returned struct's Vec
// fields. 1000 x 512 KB; pre-annotation this leaks ~512 MB.
module aggregate_return_leak

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

type Big = {
  data: Vec[UInt8];
  tag: Int;
}

#[owns_return]
fn make_big(sz: Int) -> Big {
  return Big{ data: Vec[UInt8].with_len(sz); tag: 4 };
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
    let b = make_big(sz);
    sink = sink + consume_val(b);
    i = i + 1;
  }
  io.println("aggregate return leak done sink=" + convert.int_to_string(sink));
  return 0;
}
