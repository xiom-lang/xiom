// M32: Truncation UInt64 to UInt32 (4294967295 + 1 should wrap to 0)
fn main() -> Int {
  var a: UInt64 = 4294967296;
  var b: UInt32 = a as UInt32;
  if b == 0 as UInt32 { return 0; }
  return 1;
}
