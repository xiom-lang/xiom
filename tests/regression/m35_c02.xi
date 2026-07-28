// M35-C02: if with else — two branches covering true/false
fn main() -> Int {
  var x: Int = 10;
  var y: Int = 0;
  if x > 5 { y = 1; } else { y = 2; }
  if y != 1 { return 1; }
  if x < 5 { y = 3; } else { y = 4; }
  if y != 4 { return 2; }
  return 0;
}
