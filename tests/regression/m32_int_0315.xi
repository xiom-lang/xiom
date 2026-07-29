// M32: Int16 sign extension from negative value to Int64 (sext verification)
fn main() -> Int {
  var a: Int16 = -1 as Int16;
  var b: Int64 = a as Int64;
  if b == -1 as Int64 { return 0; }
  return 1;
}
