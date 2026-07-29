// M32: let binding with narrow unsigned type
fn main() -> Int {
  var x: UInt8 = 200;
  var y: Int = x as Int;
  if y == 200 { return 0; }
  return 1;
}
