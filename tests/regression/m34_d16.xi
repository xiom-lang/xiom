// M34-D16: Tree comparison — structural equality check for two binary trees
type Node = { value: Int; left: *Node; right: *Node; }

fn tree_equal(a: *Node, b: *Node) -> Bool {
  if a == (0 as *Node) && b == (0 as *Node) { return true; }
  if a == (0 as *Node) { return false; }
  if b == (0 as *Node) { return false; }
  var va: Int;
  var vb: Int;
  unsafe { va = (*a).value; }
  unsafe { vb = (*b).value; }
  if va != vb { return false; }
  var la: *Node;
  var ra: *Node;
  var lb: *Node;
  var rb: *Node;
  unsafe { la = (*a).left; }
  unsafe { ra = (*a).right; }
  unsafe { lb = (*b).left; }
  unsafe { rb = (*b).right; }
  return tree_equal(la, lb) && tree_equal(ra, rb);
}

fn tree_identical(a: *Node, b: *Node) -> Bool {
  return a == b;
}

fn tree_both_null(a: *Node, b: *Node) -> Bool {
  return a == (0 as *Node) && b == (0 as *Node);
}

fn main() -> Int {
  var n1: *Node = 0 as *Node;
  var n2: *Node = 0 as *Node;
  if !tree_equal(n1, n2) { return 1; }
  if !tree_identical(n1, n2) { return 2; }
  if !tree_both_null(n1, n2) { return 3; }
  if !tree_equal(n1, n1) { return 4; }
  return 0;
}
