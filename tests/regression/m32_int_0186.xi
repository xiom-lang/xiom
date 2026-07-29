// M32: UInt64 comparisons
fn main() -> Int {
  var a: UInt64 = 0;
  var b: UInt64 = 18446744073709551615;
  if a <= b && b >= a && a != b { return 0; }
  return 1;
}
