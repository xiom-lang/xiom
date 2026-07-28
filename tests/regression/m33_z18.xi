// M33-Z18: Compound assignment after shadowing
fn main() -> Int {
  let a = 1;
  let a = a + 10;
  let a = a * 2;
  var x: Int = a;
  x += 3;
  x -= 5;
  x *= 2;
  if x == 40 { return 0; }
  return 1;
}
