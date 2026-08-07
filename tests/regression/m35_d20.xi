// M35-D20: AVL rotation check — compute balance factor via depth recursion
type AVLNode = { value: Int; left: *AVLNode; right: *AVLNode; }

fn avl_depth(n: *AVLNode) -> Int {
  if n == unsafe { 0 as *AVLNode } { return 0; }
  var l: *AVLNode;
  var r: *AVLNode;
  unsafe { l = (*n).left; r = (*n).right; }
  var dl: Int = avl_depth(l);
  var dr: Int = avl_depth(r);
  if dl > dr { return dl + 1; }
  return dr + 1;
}

fn avl_balance_factor(n: *AVLNode) -> Int {
  if n == unsafe { 0 as *AVLNode } { return 0; }
  var l: *AVLNode;
  var r: *AVLNode;
  unsafe { l = (*n).left; r = (*n).right; }
  return avl_depth(l) - avl_depth(r);
}

fn avl_check_invariant(n: *AVLNode) -> Bool {
  if n == unsafe { 0 as *AVLNode } { return true; }
  var bf: Int = avl_balance_factor(n);
  if bf < -1 || bf > 1 { return false; }
  var l: *AVLNode;
  var r: *AVLNode;
  unsafe { l = (*n).left; r = (*n).right; }
  return avl_check_invariant(l) && avl_check_invariant(r);
}

fn main() -> Int {
  var empty: *AVLNode = unsafe { 0 as *AVLNode };
  if avl_depth(empty) != 0 { return 1; }
  if avl_balance_factor(empty) != 0 { return 2; }
  if !avl_check_invariant(empty) { return 3; }
  return 0;
}
