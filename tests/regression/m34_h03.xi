// M34-H03: UInt->Int (unsigned to signed) — small values preserve identity
fn main() -> Int {
  var u: UInt8 = 100;
  var s: Int8 = u as Int8;
  if s == 100 as Int8 { return 0; }
  return 1;
}
