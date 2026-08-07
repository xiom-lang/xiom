// M34-D06: Nested recursive types — two-level struct hierarchy with recursive pointers
type Leaf = { data: Int; sibling: *Leaf; }
type Branch = { info: Int; first: *Leaf; next: *Branch; }

fn leaf_count(l: *Leaf) -> Int {
  if l == (unsafe { 0 as *Leaf }) { return 0; }
  var s: *Leaf;
  unsafe { s = (*l).sibling; }
  return 1 + leaf_count(s);
}

fn branch_leaf_sum(b: *Branch) -> Int {
  if b == (unsafe { 0 as *Branch }) { return 0; }
  var f: *Leaf;
  var n: *Branch;
  unsafe { f = (*b).first; }
  unsafe { n = (*b).next; }
  return leaf_count(f) + branch_leaf_sum(n);
}

fn main() -> Int {
  var empty_branch: *Branch = unsafe { 0 as *Branch };
  if branch_leaf_sum(empty_branch) != 0 { return 1; }
  var empty_leaf: *Leaf = unsafe { 0 as *Leaf };
  if leaf_count(empty_leaf) != 0 { return 2; }
  return 0;
}
