// REL-20261010-1549-pulse-4 / v0.65.2 Str retention:
// a fresh bound Str from `+` must be freed at the loop-body scope exit.
// 2M iterations x a 20-byte concat: pre-fix leaks ~40 MB (plus allocator
// overhead), so the 24 MB cap catches it; the drop keeps it flat.
// The unbound temporary form (`sink + ("x" + to_string(i)).len()`) is a
// separate, later temporary-drop item -- this probe locks the bound local.
module str_loop_leak

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

fn main() -> Int {
  let n = 2000000;
  var i = 0;
  var sink = 0;
  while i < n {
    let s = "abcdefghij" + "klmnopqrst";
    sink = sink + s.len();
    i = i + 1;
  }
  io.println("str loop leak done sink=" + convert.int_to_string(sink));
  return 0;
}
