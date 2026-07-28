// M32-X14: Combinatorial + Differential — Fibonacci while vs recursion with struct+generic+contract
type FibInput = { n: Int; }
fn fib_iter(inp: FibInput) -> Int
  requires: inp.n >= 0
  ensures: result >= 0
{
  if inp.n <= 1 { return inp.n; }
  var a: Int = 0;
  var b: Int = 1;
  var i: Int = 1;
  while i < inp.n { var t = b; b = a + b; a = t; i = i + 1; }
  return b;
}
fn fib_rec(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 0
{
  if n <= 1 { return n; }
  return fib_rec(n - 1) + fib_rec(n - 2);
}
enum FibMethod { Iter, Recurse }
fn fib(m: FibMethod, n: Int) -> Int {
  match m {
    Iter => fib_iter(FibInput{ n: n; }),
    Recurse => fib_rec(n),
  }
}
fn main() -> Int {
  var a = fib(FibMethod.Iter, 10);
  var b = fib(FibMethod.Recurse, 10);
  var c = fib(FibMethod.Iter, 7);
  var d = fib(FibMethod.Recurse, 7);
  if a == b && c == d && a == 55 && c == 13 { return 0; }
  return 1;
}
