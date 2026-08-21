// M34-W05: Shift left by 1 -- multiply by 2 on multiple types
fn main() -> Int {
  var a: Int = 7;
  var b: Int32 = 7 as Int32;
  var c: UInt = 7;
  var s1: Int = a << 1;
  var s2: Int32 = b << 1;
  var s3: UInt = c << 1;
  if s1 == 14 && s2 == 14 as Int32 && s3 == 14 { return 0; }
  return 1;
}
