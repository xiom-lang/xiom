// M32: UInt8 as pointer index offset (byte-level address)
fn main() -> Int {
  var arr: Vec[Int] = Vec[Int].new();
  arr.push(10);
  arr.push(20);
  arr.push(30);
  var idx: UInt8 = 2;
  var v: Int = arr[idx as Int];
  if v == 30 { return 0; }
  return 1;
}
