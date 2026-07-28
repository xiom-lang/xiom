// M32: Mixed-type addition Int + Int8
fn main() -> Int {
  var a: Int = 100;
  var b: Int8 = 50;
  var result: Int = a + b as Int;
  if result == 150 {
    return 0;
  }
  return 1;
}
