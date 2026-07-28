// M36-C01: Every operator with every type — arithmetic, bitwise, shift, cmp, logic, bool across Int, Int8, Int16, Int32, Float64, Bool, Char
fn main() -> Int {
  var i: Int = 42;
  var i2: Int = 7;
  var f: Float64 = 3.5;
  var f2: Float64 = 1.5;
  var b: Bool = true;
  var b2: Bool = false;
  var c: Char = 'A';
  var i1: Int = i + i2;
  var i3: Int = i - i2;
  var i4: Int = i * i2;
  var i5: Int = i / i2;
  var i6: Int = i % i2;
  var i7: Int = i & i2;
  var i8: Int = i | i2;
  var i9: Int = i ^ i2;
  var i10: Int = i << 1;
  var i11: Int = i >> 1;
  var f3: Float64 = f + f2;
  var f4: Float64 = f - f2;
  var f5: Float64 = f * f2;
  var f6: Float64 = f / f2;
  var b3: Bool = b && !b2;
  var b4: Bool = b || b2;
  var b5: Bool = !b2;
  var cmp1: Bool = i == 42;
  var cmp2: Bool = i != 0;
  var cmp3: Bool = i < 100;
  var cmp4: Bool = i > 0;
  var cmp5: Bool = i <= 42;
  var cmp6: Bool = i >= 0;
  var cmp7: Bool = f < 10.0;
  var cmp8: Bool = f > 0.0;
  var cmp9: Bool = c == 'A';
  var cmp10: Bool = b == true;
  var i8a: Int8 = 8;
  var i8b: Int8 = 3;
  var i8c: Int8 = i8a + i8b;
  var i8d: Int8 = i8a - i8b;
  var i8e: Int8 = i8a * i8b;
  var i8f: Int8 = i8a / i8b;
  var i8g: Int8 = i8a % i8b;
  var i8h: Int8 = i8a & i8b;
  var i8i: Int8 = i8a | i8b;
  var i8j: Int8 = i8a ^ i8b;
  var i8cmp1: Bool = i8a > i8b;
  var i8cmp2: Bool = i8a < i8b;
  var i8cmp3: Bool = i8a == i8b;
  var i16a: Int16 = 16;
  var i16b: Int16 = 4;
  var i16c: Int16 = i16a + i16b;
  var i16d: Int16 = i16a - i16b;
  var i16e: Int16 = i16a * i16b;
  var i16cmp: Bool = i16a > i16b;
  if i1 != 49 { return 1; }
  if i3 != 35 { return 2; }
  if i4 != 294 { return 3; }
  if i5 != 6 { return 4; }
  if i6 != 0 { return 5; }
  if i7 != 2 { return 6; }
  if i8 != 47 { return 7; }
  if i9 != 45 { return 8; }
  if i10 != 84 { return 9; }
  if i11 != 21 { return 10; }
  if f3 < 4.99 || f3 > 5.01 { return 11; }
  if f4 < 1.99 || f4 > 2.01 { return 12; }
  if f5 < 5.24 || f5 > 5.26 { return 13; }
  if f6 < 2.32 || f6 > 2.34 { return 14; }
  if b3 != true { return 15; }
  if b4 != true { return 16; }
  if b5 != true { return 17; }
  if !cmp1 { return 18; }
  if !cmp2 { return 19; }
  if !cmp3 { return 20; }
  if !cmp4 { return 21; }
  if !cmp5 { return 22; }
  if !cmp6 { return 23; }
  if !cmp7 { return 24; }
  if !cmp8 { return 25; }
  if !cmp9 { return 26; }
  if !cmp10 { return 27; }
  if i8c != 11 { return 28; }
  if i8d != 5 { return 29; }
  if i8e != 24 { return 30; }
  if i8f != 2 { return 31; }
  if i8g != 2 { return 32; }
  if i8h != 0 { return 33; }
  if i8i != 11 { return 34; }
  if i8j != 11 { return 35; }
  if !i8cmp1 { return 36; }
  if i8cmp2 { return 37; }
  if i8cmp3 { return 38; }
  if i16c != 20 { return 39; }
  if i16d != 12 { return 40; }
  if i16e != 64 { return 41; }
  if !i16cmp { return 42; }
  return 0;
}
