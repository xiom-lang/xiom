// M35-C21: match on Int -- integer value dispatch with multiple arms
fn classify_value(x: Int) -> Int {
  match x {
    0 => 0,
    1 => 1,
    2 => 4,
    3 => 9,
    _ => x * x,
  }
}
fn main() -> Int {
  if classify_value(0) != 0 { return 1; }
  if classify_value(1) != 1 { return 2; }
  if classify_value(2) != 4 { return 3; }
  if classify_value(3) != 9 { return 4; }
  if classify_value(5) != 25 { return 5; }
  return 0;
}
