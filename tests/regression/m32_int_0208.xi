// M32: Vec Int16 elements
fn main() -> Int {
  var v: Vec[Int16] = Vec[Int16].new();
  v.push(-32768 as Int16);
  v.push(32767);
  if v[0] == -32768 as Int16 { return 0; }
  return 1;
}
