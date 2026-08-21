// M33-K16: Closure in while loop -- accumulator pattern
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  var acc = |x| x + 1;
  while i < 5 { sum = sum + i; i = acc(i); }
  if sum != 10 { return 1; }
  return 0;
}
