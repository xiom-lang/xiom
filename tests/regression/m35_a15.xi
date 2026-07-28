// M35-A15: Sum of digits — compute sum of decimal digits via while loop
fn digit_sum(n: Int) -> Int {
  var x: Int = n;
  var s: Int = 0;
  while x > 0 {
    s = s + x % 10;
    x = x / 10;
  }
  return s;
}
fn main() -> Int {
  if digit_sum(12345) != 15 { return 1; }
  if digit_sum(0) != 0 { return 2; }
  if digit_sum(99999) != 45 { return 3; }
  if digit_sum(10001) != 2 { return 4; }
  if digit_sum(8765) != 26 { return 5; }
  return 0;
}
