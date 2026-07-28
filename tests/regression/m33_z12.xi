// M33-Z12: Compound assignment with var initial assignment
fn main() -> Int {
  var base: Int = 20;
  var x: Int = base;
  x += 15;
  x -= 5;
  x *= 3;
  x /= 6;
  x %= 4;
  if x == 3 { return 0; }
  return 1;
}
