// M34-D03: Tree count — recursive count of all nodes in binary tree
type Node = { value: Int; left: *Node; right: *Node; }

fn node_count(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return 1 + node_count(l) + node_count(r);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if node_count(empty) != 0 { return 1; }
  return 0;
}
