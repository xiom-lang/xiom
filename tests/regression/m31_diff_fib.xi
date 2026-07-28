// Combinatorial stress: 3 different Fibonacci implementations must agree
fn fib_rec(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib_rec(n - 1) + fib_rec(n - 2);
}
fn fib_iter(n: Int) -> Int {
  if n <= 1 { return n; }
  var a: Int = 0;
  var b: Int = 1;
  var i: Int = 1;
  while i < n {
    var t = b;
    b = a + b;
    a = t;
    i = i + 1;
  }
  return b;
}
fn main() -> Int {
  var a = fib_iter(10);
  var b = fib_rec(10);
  if a == b && a == 55 { return 0; }
  return 1;
}
