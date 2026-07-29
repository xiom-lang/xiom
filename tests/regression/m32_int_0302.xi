// M32: Vec UInt8 elements with high values
fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8].new();
  v.push(255);
  v.push(128);
  if v[0] == 255 as UInt8 && v[1] == 128 as UInt8 { return 0; }
  return 1;
}
