// M32: Int8 sign-test: negative >> 7 must = -1, positive >> 7 must = 0
fn main() -> Int {
  var neg: Int8 = -128 as Int8;
  var pos: Int8 = 127;
  var rn: Int8 = neg >> 7;
  var rp: Int8 = pos >> 7;
  if rn == -1 as Int8 && rp == 0 as Int8 { return 0; }
  return 1;
}
