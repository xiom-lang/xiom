// M32: Int64 add wraparound (max + 1 = min)
fn main() -> Int {
  var a: Int64 = 9223372036854775807;
  var b: Int64 = 1;
  var c: Int64 = a + b;
  if c < 0 { return 0; }
  return 1;
}
