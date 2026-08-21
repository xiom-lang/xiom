// M34-D01: Binary tree -- struct with left/right recursive pointers
type Node = { value: Int; left: *Node; right: *Node; }

fn tree_sum(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return val + tree_sum(l) + tree_sum(r);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if tree_sum(empty) != 0 { return 1; }
  return 0;
}
