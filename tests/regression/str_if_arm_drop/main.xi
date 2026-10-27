// v0.65.2 -> v0.65.3: a fresh Str bound in an if-arm must capture its
// pointer at the binding; the scope-exit drop must not load the if-arm
// alloca from a non-dominating merge/loop exit (engine sfr_unique).
module str_if_arm_drop

use xiom.io;
use xiom.convert;
use xiom.convert.parse;

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 1000 {
    if i % 2 == 0 {
      let s = "abcdefghij" + "klmnopqrst";
      sink = sink + s.len();
    }
    i = i + 1;
  }
  io.println("str if-arm drop done sink=" + convert.int_to_string(sink));
  return 0;
}
