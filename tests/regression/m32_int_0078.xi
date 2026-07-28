// M32: Compound arithmetic expression
fn main() -> Int {
  var result: Int = (10 + 20) * (30 - 15) / 5;
  if result == 90 {
    return 0;
  }
  return 1;
}
