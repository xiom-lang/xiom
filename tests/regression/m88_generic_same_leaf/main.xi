module m88.main

use m88.generics as g;
use m88.hard as h;

fn main() -> Int {
  var plain = g.make_plain();
  if g.plain_value(&plain) != 7 { return 1; }

  var packed = h.make_packed();
  if h.packed_item(&packed) != 42 { return 2; }

  return 0;
}
