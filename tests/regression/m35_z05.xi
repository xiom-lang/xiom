// M35-Z05: recursion+enum+match+contract+module+closure+array+const+cast+while
const LIMIT: Int = 10;
enum Tree { Leaf(v: Int), Branch(l: Int, r: Int) }
fn sum_tree(t: Tree, max_depth: Int) -> Int
  requires: max_depth >= 0
  ensures: result >= 0
{
  if max_depth == 0 { return 0; }
  match t {
    Leaf(v) => v,
    Branch(l, r) => sum_rec(l, max_depth - 1) + sum_rec(r, max_depth - 1),
  }
}
fn sum_rec(d: Int, limit: Int) -> Int {
  if limit <= 0 || d <= 0 { return d; }
  return d + sum_rec(d - 1, limit - 1);
}
module forest {
  pub fn eval(t: Tree, d: Int) -> Int { return sum_tree(t, d); }
  pub fn identity(x: Int) -> Int { return x; }
}
use forest.eval;
fn main() -> Int {
  var t1 = Tree.Leaf(7);
  var t2 = Tree.Branch(3, 4);
  var arr = [1, 2, 3];
  var f = |x| x + 1;
  var r1 = eval(t1, 10);
  var r2 = eval(t2, 10);
  var acc = arr[0] + arr[1] + arr[2];
  if r1 == 7 && r2 == 16 && f(acc) == 7 { return 0; }
  return 1;
}
