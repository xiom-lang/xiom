// REL-20261028: replaced-Str-buffer reclamation. `s = s + chunk` allocates a
// new buffer each iteration; the old owned buffer must be freed (pre-fix this
// grew ~116 MB for 5000 small appends; now flat).
module str_rebind_growth

use xiom.io;
use xiom.convert;

fn build(n: Int) -> Str {
  var s = "";
  var i = 0;
  while i < n {
    s = s + "0123456789";
    i = i + 1;
  }
  return s;
}

fn main() -> Int {
  let s = build(5000);
  if s.len() != 50000 { return 1; }
  io.println("str rebind growth ok len=" + convert.int_to_string(s.len()));
  return 0;
}
