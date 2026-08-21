// M35-D01: Stack -- push/pop/peek with array
fn main() -> Int {
  var data0: Int = 0;
  var data1: Int = 0;
  var data2: Int = 0;
  var data3: Int = 0;
  var top: Int = -1;
  top = 0;
  data0 = 10;
  top = 1;
  data1 = 20;
  top = 2;
  data2 = 30;
  if data2 != 30 { return 1; }
  top = 1;
  if data1 != 20 { return 2; }
  top = 0;
  if data0 != 10 { return 3; }
  top = -1;
  if top != -1 { return 4; }
  return 0;
}
