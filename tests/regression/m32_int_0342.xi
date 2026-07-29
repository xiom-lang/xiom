// M32: Int64 narrowing to Int32 (truncation)
fn main() -> Int {
  var a: Int64 = 4294967296;
  var b: Int32 = a as Int32;
  if b == 0 as Int32 { return 0; }
  return 1;
}
