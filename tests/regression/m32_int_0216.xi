// M32: Function chain narrow params through multiple calls
fn inc8(x: Int8) -> Int8 { return x + 1 as Int8; }
fn dec8(x: Int8) -> Int8 { return x - 1 as Int8; }
fn main() -> Int {
  var a: Int8 = inc8(127);
  var b: Int8 = dec8(-128 as Int8);
  if a == -128 as Int8 && b == 127 as Int8 { return 0; }
  return 1;
}
