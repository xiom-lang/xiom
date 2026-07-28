// M34-D08: Recursive struct in enum payload — enum variant contains pointer to recursive struct
type Node = { value: Int; left: *Node; right: *Node; }

enum TreeOp {
  Leaf,
  Branch(n: *Node),
}

fn node_depth(n: *Node) -> Int {
  if n == (0 as *Node) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return 1 + node_depth(l) + node_depth(r);
}

fn op_exec(op: TreeOp) -> Int {
  match op {
    TreeOp.Leaf => 0,
    TreeOp.Branch(n) => node_depth(n),
  }
}

fn main() -> Int {
  var leaf_op = TreeOp.Leaf;
  if op_exec(leaf_op) != 0 { return 1; }
  return 0;
}
