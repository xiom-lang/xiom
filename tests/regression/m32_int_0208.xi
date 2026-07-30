// M32: Vec[Int16] elements via explicit casts
fn main() -> Int {
  var v: Vec[Int16] = Vec[Int16].new();
  v.push(-32768 as Int16);
  v.push(32767 as Int16);
  if v[0] == -32768 as Int16 && v[1] == 32767 as Int16 { return 0; }
  return 1;
}
