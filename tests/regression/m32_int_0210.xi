// M32: Vec push and read narrow int with wraparound
fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8].new();
  v.push(255);
  var x: UInt8 = v[0] + 1 as UInt8;
  if x == 0 as UInt8 { return 0; }
  return 1;
}
