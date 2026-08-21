// M34-J17: Module with invariant -- invariant types defined inside modules
module non_zero {
  pub type Positive = { val: Int; invariant: val > 0; }
  pub fn make_pos(v: Int) -> Positive { return Positive{ val: v; }; }
  pub fn get(p: Positive) -> Int { return p.val; }
  pub fn add(a: Positive, b: Positive) -> Positive {
    return Positive{ val: a.val + b.val; };
  }
}
module bounded {
  pub type Bounded = { val: Int; min: Int; max: Int; invariant: val >= min && val <= max; }
  pub fn make_bounded(v: Int, lo: Int, hi: Int) -> Bounded {
    return Bounded{ val: v; min: lo; max: hi; };
  }
}
use non_zero.make_pos;
use non_zero.add;
use non_zero.get;
use bounded.make_bounded;
fn main() -> Int {
  var p1 = make_pos(5);
  var p2 = make_pos(10);
  var sum = add(p1, p2);
  if get(sum) != 15 { return 1; }
  var b = make_bounded(7, 1, 10);
  if b.val != 7 { return 2; }
  return 0;
}
