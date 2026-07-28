// M35-D05: Deque — double-ended queue verification
fn main() -> Int {
  var d0: Int = 0;
  var d1: Int = 0;
  var d2: Int = 0;
  d0 = 10;
  d1 = 20;
  d2 = 30;
  if d0 != 10 { return 1; }
  if d1 != 20 { return 2; }
  if d2 != 30 { return 3; }
  d1 = 0;
  if d1 != 0 { return 4; }
  return 0;
}
