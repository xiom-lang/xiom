// M32: Int8 sign extension from positive to Int (sext: 127 stays 127)
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int = a as Int;
  if b == 127 { return 0; }
  return 1;
}
