// M34-W01: AND identity — x & -1 == x on signed integer types
// Avoid -1 cast to small types (codegen limitation)
fn main() -> Int {
  var a: Int = 42;
  var a32: Int32 = 42 as Int32;
  var a64: Int64 = 42 as Int64;
  var b: Int = a & (-1 as Int);
  var b32: Int32 = a32 & (0xFFFFFFFF as Int32);
  var b64: Int64 = a64 & (0xFFFFFFFFFFFFFFFF as Int64);
  if b == 42 && b32 == 42 as Int32 && b64 == 42 as Int64 { return 0; }
  return 1;
}
