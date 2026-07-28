// M35-D07: Min-heap — bubble-up verify
fn main() -> Int {
  var d0: Int = 10;
  var d1: Int = 20;
  var d2: Int = 30;
  if d0 > d1 { return 1; }
  if d0 > d2 { return 2; }
  if d1 > d2 { return 3; }
  return 0;
}
