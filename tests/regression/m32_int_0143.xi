// M32: Int64 mul wraparound (large * 2)
fn main() -> Int {
  var a: Int64 = 4611686018427387904;
  var b: Int64 = 2;
  var c: Int64 = a * b;
  if c < 0 { return 0; }
  return 1;
}
