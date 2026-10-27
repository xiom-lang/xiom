// REL-20261027-1717-engine: a fresh Vec passed by & to an ordinary
// (non-retaining) XIOM function must still be reclaimed at the caller's
// scope. 1000 x 512 KB; the v0.65.1/v0.65.2 pass marks &v escaping and
// leaks ~512 MB (killed at the 1 GB cap).
module borrow_vec_leak

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

fn consume(v: &Vec[UInt8]) -> Int {
  return v.len();
}

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var v = Vec[UInt8].with_len(sz);
    sink = sink + consume(&v);
    i = i + 1;
  }
  io.println("borrow vec leak done sink=" + convert.int_to_string(sink));
  return 0;
}
