// M34-D04: Tree depth — compute max depth of binary tree
type Node = { value: Int; left: *Node; right: *Node; }

fn max2(a: Int, b: Int) -> Int {
  if a > b { return a; }
  return b;
}

fn tree_depth(n: *Node) -> Int {
  if n == (0 as *Node) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return 1 + max2(tree_depth(l), tree_depth(r));
}

fn main() -> Int {
  var empty: *Node = 0 as *Node;
  if tree_depth(empty) != 0 { return 1; }
  return 0;
}
