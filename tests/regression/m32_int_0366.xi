// M32: Int8 min + Int8 min = 0 (wraparound: -128 + -128 = 0)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var c: Int8 = a + a;
  if c == 0 as Int8 { return 0; }
  return 1;
}
