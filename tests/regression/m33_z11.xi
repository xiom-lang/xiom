// M33-Z11: Compound assignment in if body
fn main() -> Int {
  var x: Int = 5;
  if x > 0 {
    x += 10;
    x *= 2;
    x -= 3;
  } else {
    x = 0;
  }
  if x == 27 { return 0; }
  return 1;
}
