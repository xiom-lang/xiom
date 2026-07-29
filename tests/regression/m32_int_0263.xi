// M32: UInt32(4000000000) as UInt64 preserves value
fn main() -> Int {
  var a: UInt32 = 4000000000;
  var b: UInt64 = a as UInt64;
  if b == 4000000000 as UInt64 { return 0; }
  return 1;
}
