// M32: Vec Int8 elements
fn main() -> Int {
  var v: Vec[Int8] = Vec[Int8].new();
  v.push(127);
  v.push(-128 as Int8);
  if v.len() == 2 { return 0; }
  return 1;
}
