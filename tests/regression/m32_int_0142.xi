// M32: Int64 sub wraparound (min - 1 = max)
fn main() -> Int {
  var a: Int64 = -9223372036854775808 as Int64;
  var b: Int64 = 1;
  var c: Int64 = a - b;
  if c > 0 { return 0; }
  return 1;
}
