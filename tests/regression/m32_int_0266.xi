// M32: UInt16(60000) as Int32 then as Int64: chain preserves unsigned
fn main() -> Int {
  var a: UInt16 = 60000;
  var b: Int32 = a as Int32;
  var c: Int64 = b as Int64;
  if c == 60000 as Int64 { return 0; }
  return 1;
}
