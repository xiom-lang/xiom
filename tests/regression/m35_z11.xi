// M35-Z11: recursion+generic+struct+enum+match+Option+Result+contract+compound_assign
type Node = { val: Int; next: Option[Int]; }
enum List { Nil, Cons(hd: Int, tl: Int) }
fn list_len_rec(n: Int) -> Int {
  if n <= 0 { return 0; }
  return 1 + list_len_rec(n - 1);
}
fn list_len(l: List) -> Int
  requires: l != List.Nil
  ensures: result >= 0
{
  match l { Nil => 0, Cons(_, tl) => 1 + list_len_rec(tl) }
}
fn list_sum_rec(n: Int) -> Int {
  if n <= 0 { return 0; }
  return n + list_sum_rec(n - 1);
}
fn list_sum[T](l: List) -> Int
  ensures: result >= 0
{
  match l { Nil => 0, Cons(hd, tl) => hd + list_sum_rec(tl) }
}
fn to_result(n: Int) -> Result[Int, Str] {
  if n > 100 { return Err("big"); }
  return Ok(n);
}
module list_ops {
  pub fn len(l: List) -> Int { return list_len(l); }
  pub fn sum(l: List) -> Int { return list_sum(l); }
  pub fn try_wrap(n: Int) -> Result[Int, Str] { return to_result(n); }
}
use list_ops.len;
use list_ops.sum;
use list_ops.try_wrap;
fn main() -> Int {
  var l1 = List.Cons(3, 3);
  var l2 = List.Cons(5, 5);
  var len1 = len(l1);
  var len2 = len(l2);
  var s1 = sum(l1);
  var s2 = sum(l2);
  var total = s1 + s2;
  match try_wrap(total) {
    Ok(v) => { if v == 29 && len1 == 4 && len2 == 6 { return 0; } }
    Err(_) => { return 1; }
  }
  return 2;
}
