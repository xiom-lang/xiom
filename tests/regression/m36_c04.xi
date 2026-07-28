// M36-C04: Every generic pattern — identity, swap, struct generic, multi-param, enum generic
fn identity[T](x: T) -> T { return x; }
fn pair[T, U](a: T, b: U) -> T { return a; }
fn triple[T1, T2, T3](a: T1, b: T2, c: T3) -> T1 { return a; }
type Pair[T] = { left: T; right: T; }
fn make_pair[T](l: T, r: T) -> Pair[T] { return Pair[T]{ left: l; right: r; }; }
fn sum_pair(p: Pair[Int]) -> Int { return p.left + p.right; }
fn double[T](x: T, y: T) -> T { if x == x { return x; } return y; }
fn quad[T, U, V, W](a: T, b: U, c: V, d: W) -> T { return a; }
fn quintuple[A, B, C, D, E](a: A, b: B, c: C, d: D, e: E) -> A { return a; }
fn main() -> Int {
  if identity(42) != 42 { return 1; }
  if identity(true) != true { return 2; }
  if identity("hi") != "hi" { return 3; }
  if pair(1, "two") != 1 { return 4; }
  if pair(true, 99) != true { return 5; }
  if triple(10, 20, 30) != 10 { return 6; }
  if triple("a", 'b', true) != "a" { return 7; }
  var p = make_pair(5, 9);
  if p.left != 5 { return 8; }
  if p.right != 9 { return 9; }
  if sum_pair(p) != 14 { return 10; }
  if double(42, 99) != 42 { return 11; }
  if double(true, false) != true { return 12; }
  if quad(1, 2.0, 'c', "d") != 1 { return 13; }
  if quintuple(10, 20, 30, 40, 50) != 10 { return 14; }
  return 0;
}
