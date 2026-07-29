// M32: UInt32 cast from Int32 negative
fn main() -> Int {
  var a: Int32 = -1 as Int32;
  var b: UInt32 = a as UInt32;
  if b == 4294967295 as UInt32 { return 0; }
  return 1;
}
