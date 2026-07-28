// M35-D03: Queue — enqueue/dequeue using array
fn main() -> Int {
  var data0: Int = 0;
  var data1: Int = 0;
  var data2: Int = 0;
  var head: Int = 0;
  var tail: Int = 0;
  data0 = 1;
  tail = 1;
  data1 = 2;
  tail = 2;
  data2 = 3;
  tail = 3;
  if data0 != 1 { return 1; }
  head = 1;
  if data1 != 2 { return 2; }
  head = 2;
  if data2 != 3 { return 3; }
  return 0;
}
