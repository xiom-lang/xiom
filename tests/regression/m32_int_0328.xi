// M32: Bool condition with narrow int (Int8 comparison to Bool)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 127;
  var result: Bool = a < b;
  if result { return 0; }
  return 1;
}
