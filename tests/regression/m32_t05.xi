// M32-T05: Char to Int -- as cast from Char to Int
fn main() -> Int {
  var a: Int = 'A' as Int;
  var z: Int = 'Z' as Int;
  var zero: Int = '0' as Int;
  if a == 65 && z == 90 && zero == 48 { return 0; }
  return 1;
}
