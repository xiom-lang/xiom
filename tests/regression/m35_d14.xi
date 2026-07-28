// M35-D14: Ring buffer — oldest overwritten
fn main() -> Int {
  var b0: Int = 10;
  var b1: Int = 20;
  var b2: Int = 30;
  var b3: Int = 40;
  var head: Int = 1;
  b0 = 50;
  if b0 != 50 { return 1; }
  head = 2;
  if b2 != 30 { return 2; }
  head = 3;
  b1 = 60;
  if b1 != 60 { return 3; }
  head = 0;
  if b0 != 50 { return 4; }
  return 0;
}
