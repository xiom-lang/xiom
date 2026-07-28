// M33-Z20: Compound assignment with Int8/Int16/Int32/Int64 and UInt types
fn main() -> Int {
  var a: Int8 = 10;
  var b: Int16 = 100;
  var c: Int32 = 1000;
  var d: Int64 = 10000;
  var e: UInt8 = 50;
  var f: UInt16 = 500;
  a += 5;
  b -= 20;
  c *= 3;
  d /= 2;
  e += 25;
  f -= 100;
  if a == 15 as Int8 && b == 80 as Int16 && c == 3000 as Int32 &&
     d == 5000 as Int64 && e == 75 as UInt8 && f == 400 as UInt16 {
    return 0;
  }
  return 1;
}
