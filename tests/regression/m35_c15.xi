// M35-C15: loop with accumulator — building a value across loop iterations
fn factorial(n: Int) -> Int {
  var acc: Int = 1;
  var i: Int = 1;
  while i <= n { acc = acc * i; i = i + 1; }
  return acc;
}
fn main() -> Int {
  if factorial(0) != 1 { return 1; }
  if factorial(1) != 1 { return 2; }
  if factorial(5) != 120 { return 3; }
  if factorial(6) != 720 { return 4; }
  return 0;
}
