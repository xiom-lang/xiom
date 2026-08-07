// M34-D13: Tree fold — accumulation across binary tree with null pointers
type Node = { value: Int; left: *Node; right: *Node; }

fn fold_sum(n: *Node, acc: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return acc; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return fold_sum(r, fold_sum(l, acc + val));
}

fn fold_count(n: *Node, acc: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return acc; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return fold_count(r, fold_count(l, acc + 1));
}

fn fold_max(n: *Node, best: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return best; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  var cur: Int = best;
  if val > cur { cur = val; }
  var left_best: Int = fold_max(l, cur);
  if left_best > cur { cur = left_best; }
  return fold_max(r, cur);
}

fn main() -> Int {
  var n: *Node = unsafe { 0 as *Node };
  if fold_sum(n, 0) != 0 { return 1; }
  if fold_sum(n, 10) != 10 { return 2; }
  if fold_count(n, 0) != 0 { return 3; }
  if fold_max(n, -1) != -1 { return 4; }
  return 0;
}
