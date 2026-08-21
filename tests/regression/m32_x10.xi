// M32-X10: Combinatorial + Differential -- identity transform equivalence with generic+enum+contract
enum TransformKind { AddOne, MulTwo, Negate }
fn apply(kind: TransformKind, x: Int) -> Int
  ensures: result != x || kind == TransformKind.AddOne
{
  match kind {
    AddOne => x + 1,
    MulTwo => x * 2,
    Negate => -x,
  }
}
fn compose_twice(kind: TransformKind, x: Int) -> Int {
  var y = apply(kind, x);
  var z = apply(kind, y);
  return z;
}
fn main() -> Int {
  var r1 = compose_twice(TransformKind.AddOne, 5);
  var r2 = 5 + 2;
  var r3 = compose_twice(TransformKind.MulTwo, 3);
  var r4 = 3 * 4;
  var r5 = compose_twice(TransformKind.Negate, 7);
  var r6 = 7;
  if r1 == r2 && r3 == r4 && r5 == r6 && r1 == 7 && r3 == 12 { return 0; }
  return 1;
}
