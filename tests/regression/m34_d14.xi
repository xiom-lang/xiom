// M34-D14: Tree map — value transformation across binary tree nodes
type Node = { value: Int; left: *Node; right: *Node; }

fn tree_double_values(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return val * 2 + tree_double_values(l) + tree_double_values(r);
}

fn tree_negate_values(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return -val + tree_negate_values(l) + tree_negate_values(r);
}

fn tree_add_const(n: *Node, c: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return (val + c) + tree_add_const(l, c) + tree_add_const(r, c);
}

fn main() -> Int {
  var n: *Node = unsafe { 0 as *Node };
  if tree_double_values(n) != 0 { return 1; }
  if tree_negate_values(n) != 0 { return 2; }
  if tree_add_const(n, 5) != 0 { return 3; }
  return 0;
}
