// M36-S04: Parser tree traversal -- pre-order and post-order walk
type Tree = { value: Int; left: Int; right: Int; }
fn make_tree(v: Int, l: Int, r: Int) -> Tree {
  return Tree{ value: v; left: l; right: r; };
}
fn preorder_sum(node: Tree, left: Tree, right: Tree) -> Int {
  var s: Int = node.value;
  if node.left > 0 { s = s + left.value; }
  if node.right > 0 { s = s + right.value; }
  return s;
}
fn postorder_max(node: Tree, left: Tree, right: Tree) -> Int {
  var m: Int = node.value;
  if node.left > 0 { if left.value > m { m = left.value; } }
  if node.right > 0 { if right.value > m { m = right.value; } }
  return m;
}
fn walk_depth(node: Tree, left: Tree, right: Tree) -> Int {
  var depth: Int = 0;
  if node.left > 0 { depth = depth + 1; }
  if node.right > 0 { depth = depth + 1; }
  return depth;
}
fn main() -> Int {
  var root = make_tree(10, 1, 1);
  var l = make_tree(5, 0, 0);
  var r = make_tree(15, 0, 0);
  var sum = preorder_sum(root, l, r);
  if sum != 30 { return 1; }
  var mx = postorder_max(root, l, r);
  if mx != 15 { return 2; }
  var d = walk_depth(root, l, r);
  if d != 2 { return 3; }
  var leaf = make_tree(7, 0, 0);
  var d2 = walk_depth(leaf, leaf, leaf);
  if d2 != 0 { return 4; }
  return 0;
}
