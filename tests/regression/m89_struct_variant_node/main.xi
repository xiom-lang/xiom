module m89.main

use m89.types;
use m89.tree;

fn main() -> Int {
  var t = m89.tree.build();
  if m89.tree.root_value(&t) != 1 { return 1; }

  var n = m89.types.make_node(2);
  if n.value != 2 { return 2; }
  return 0;
}
