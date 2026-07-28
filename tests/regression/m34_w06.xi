// M34-W06: Shift right by 1 — divide by 2 (signed) on multiple types
fn main() -> Int {
  var a: Int = 128;
  var b: Int32 = 128 as Int32;
  var c: UInt = 128;
  var s1: Int = a >> 1;
  var s2: Int32 = b >> 1;
  var s3: UInt = c >> 1;
  if s1 == 64 && s2 == 64 as Int32 && s3 == 64 { return 0; }
  return 1;
}
