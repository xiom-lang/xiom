// m375 witness: Result[Vec[UInt8], IOError] payload bound by match and
// consumed in a loop. The Vec buffer is fresh but the match binding is not a
// `let` Vec local, so the drop pass never sees it.
module read_at_loop_leak

use xiom.io;
use xiom.convert;

fn main() -> Int {
  let path = "docs/COMPILER_BUGS.md";
  var i = 0;
  var sink = 0;
  while i < 10000 {
    let r = io.read_at(path, 0, 4096);
    match r {
      Ok(v) => { sink = sink + v.len(); }
      Err(_e) => { return 2; }
    }
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  return 0;
}
