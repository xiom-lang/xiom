// M35-Z28: deep_recursion+generic+enum+struct+contract+module+derive+match+while+Option+compound_assign
type Node = { val: Int; left: Int; right: Int; } derive[Eq]
enum Order { Pre, In, Post }
fn tree_depth(n: Int) -> Int {
  if n <= 0 { return 0; }
  var l = tree_depth(n - 1);
  var r = tree_depth(n - 2);
  if l >= r { return l + 1; }
  return r + 1;
}
fn visit[T](n: Node, order: Order) -> Int
  requires: n.val >= 0
  ensures: result >= 0
{
  match order {
    Pre => n.val + n.left + n.right,
    In => n.left + n.val + n.right,
    Post => n.left + n.right + n.val,
  }
}
module tree {
  pub fn depth(n: Int) -> Int { return tree_depth(n); }
  pub fn sum(n: Node, o: Order) -> Int { return visit(n, o); }
  pub fn max_val(a: Int, b: Int) -> Int { if a >= b { return a; } return b; }
}
use tree.depth;
use tree.sum;
use tree.max_val;
fn main() -> Int {
  var n = Node{ val: 10; left: 5; right: 3; };
  var p1 = sum(n, Order.Pre);
  var p2 = sum(n, Order.In);
  var p3 = sum(n, Order.Post);
  var d = depth(5);
  var chk = 0;
  if p1 == 18 { chk += 1; }
  if p2 == 18 { chk += 1; }
  if p3 == 18 { chk += 1; }
  if d >= 2 { chk += 1; }
  var mx = max_val(d, 10);
  if mx == 10 { chk += 1; }
  if chk == 5 { return 0; }
  return 1;
}
