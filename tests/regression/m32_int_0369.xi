// M32: UInt16 decrement past 0 wraps to 65535 multiple times
fn main() -> Int {
  var a: UInt16 = 3;
  var b: UInt16 = a - 1 as UInt16;
  var c: UInt16 = b - 1 as UInt16;
  var d: UInt16 = c - 1 as UInt16;
  var e: UInt16 = d - 1 as UInt16;
  if e == 65535 as UInt16 { return 0; }
  return 1;
}
