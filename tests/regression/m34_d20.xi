// M34-D20: Recursive invariant + serialization pattern — depth-bounded tree walk with value collection
type Node = { value: Int; left: *Node; right: *Node; }

fn tree_depth(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  var dl: Int = tree_depth(l);
  var dr: Int = tree_depth(r);
  if dl > dr { return dl + 1; }
  return dr + 1;
}

fn depth_invariant_holds(n: *Node, max_remaining: Int) -> Bool {
  if n == (unsafe { 0 as *Node }) { return true; }
  if max_remaining <= 0 { return false; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return depth_invariant_holds(l, max_remaining - 1) && depth_invariant_holds(r, max_remaining - 1);
}

fn tree_node_count(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return 1 + tree_node_count(l) + tree_node_count(r);
}

fn tree_leaf_count(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  if l == (unsafe { 0 as *Node }) && r == (unsafe { 0 as *Node }) { return 1; }
  return tree_leaf_count(l) + tree_leaf_count(r);
}

fn preorder_size(n: *Node) -> Int {
  return tree_node_count(n);
}

fn main() -> Int {
  var n: *Node = unsafe { 0 as *Node };
  if tree_depth(n) != 0 { return 1; }
  if !depth_invariant_holds(n, 0) { return 2; }
  if !depth_invariant_holds(n, 5) { return 3; }
  if tree_node_count(n) != 0 { return 4; }
  if tree_leaf_count(n) != 0 { return 5; }
  if preorder_size(n) != 0 { return 6; }
  return 0;
}
