// REL-20261028: nested struct Vec fields. `Outer{ inner: Inner{ data: Vec }}`
// pushed into a container must free the nested Vec payload. Pre-fix this
// leaked 431 MB per 1000 iterations of 512 KB.
module container_nested_struct

use xiom.io;
use xiom.convert;

type Inner = { data: Vec[UInt8]; }
type Outer = { inner: Inner; id: Int; }

fn main() -> Int {
  let n = 1000;
  let sz = 512000;
  var i = 0;
  var sink = 0;
  while i < n {
    var os = Vec[Outer].new();
    var xs = Vec[UInt8].with_len(sz);
    var o = Outer{ inner: Inner{ data: xs }; id: i };
    os.push(o);
    // Direct scalar element reads stay allowed.
    sink = sink + os[0].id;
    i = i + 1;
  }
  io.println("nested struct done sink=" + convert.int_to_string(sink));
  return 0;
}
