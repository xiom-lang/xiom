// M32-X05: Combinatorial + Differential -- add-then-mul vs mul-then-add with contract+generic
enum Op { AddFirst, MulFirst }
fn compute(op: Op, a: Int, b: Int, c: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  requires: c >= 0
  ensures: result >= 0
{
  match op {
    AddFirst => (a + b) * c,
    MulFirst => (a * c) + (b * c),
  }
}
fn main() -> Int {
  var r1 = compute(Op.AddFirst, 3, 5, 7);
  var r2 = compute(Op.MulFirst, 3, 5, 7);
  var r3 = compute(Op.AddFirst, 10, 2, 4);
  var r4 = compute(Op.MulFirst, 10, 2, 4);
  if r1 == r2 && r3 == r4 && r1 == 56 { return 0; }
  return 1;
}
