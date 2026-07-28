// M32-L10: Infinite loop with break guard — while true { ... }
fn main() -> Int {
  var sum: Int = 0;
  var n: Int = 1;
  while true {
    sum += n;
    n += 1;
    if sum > 50 { break; }
  }
  // 1+2+3+4+5+6+7+8+9+10 = 55 (> 50), n=11
  if sum == 55 && n == 11 { return 0; }
  return 1;
}
