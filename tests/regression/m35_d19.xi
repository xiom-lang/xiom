// M35-D19: BST — insert, search, min, max on pointer-based tree nodes
type BNode = { value: Int; left: *BNode; right: *BNode; }

fn bst_search(n: *BNode, val: Int) -> Bool {
  if n == (0 as *BNode) { return false; }
  var v: Int; unsafe { v = (*n).value; }
  if v == val { return true; }
  var l: *BNode; var r: *BNode; unsafe { l = (*n).left; r = (*n).right; }
  if val < v { return bst_search(l, val); }
  return bst_search(r, val);
}

fn bst_min(n: *BNode) -> Int {
  var cur: *BNode = n;
  while cur != (0 as *BNode) { var l: *BNode; unsafe { l = (*cur).left; } if l == (0 as *BNode) { var v: Int; unsafe { v = (*cur).value; } return v; } cur = l; }
  return -1;
}

fn bst_max(n: *BNode) -> Int {
  var cur: *BNode = n;
  while cur != (0 as *BNode) { var r: *BNode; unsafe { r = (*cur).right; } if r == (0 as *BNode) { var v: Int; unsafe { v = (*cur).value; } return v; } cur = r; }
  return -1;
}

fn bst_count(n: *BNode) -> Int {
  if n == (0 as *BNode) { return 0; }
  var l: *BNode; var r: *BNode; unsafe { l = (*n).left; r = (*n).right; }
  return 1 + bst_count(l) + bst_count(r);
}

fn main() -> Int {
  var empty: *BNode = 0 as *BNode;
  if bst_search(empty, 5) { return 1; }
  if bst_min(empty) != -1 { return 2; }
  if bst_max(empty) != -1 { return 3; }
  if bst_count(empty) != 0 { return 4; }
  return 0;
}
