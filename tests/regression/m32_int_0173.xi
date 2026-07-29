// M32: UInt32 sub wraparound (0 - 1 = 4294967295)
fn main() -> Int {
  var a: UInt32 = 0;
  var b: UInt32 = 1;
  var c: UInt32 = a - b;
  if c == 4294967295 as UInt32 { return 0; }
  return 1;
}
