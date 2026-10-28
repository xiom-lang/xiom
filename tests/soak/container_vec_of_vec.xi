// REL-20261028: Vec[Vec[UInt8]] element drops. A push-only container built
// from owned inner Vecs must free each element's inner buffer. Pre-fix this
// leaked 505 MB per 1000 iterations of 512 KB.
module container_vec_of_vec

use xiom.io;
use xiom.convert;

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var outer = Vec[Vec[UInt8]].new();
    var inner = Vec[UInt8].with_len(sz);
    outer.push(inner);
    sink = sink + outer[0].len();
    i = i + 1;
  }
  io.println("vec-of-vec done sink=" + convert.int_to_string(sink));
  return 0;
}
