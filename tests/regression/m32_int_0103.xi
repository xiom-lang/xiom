// M32: Int8 neg of min value (-(-128) wraps to -128)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = -a;
  if b == -128 as Int8 { return 0; }
  return 1;
}
