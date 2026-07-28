// M35-D27: Skip list — multi-level sorted verification
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 30;
  var c: Int = 50;
  var d: Int = 70;
  if a > b { return 1; }
  if b > c { return 2; }
  if c > d { return 3; }
  if a != 10 { return 4; }
  if d != 70 { return 5; }
  return 0;
}
