// M32-S14: Struct with mixed numeric types (Int8, Int16, Int32, Int64, Float64)
type Mixed = { a: Int8; b: Int16; c: Int32; d: Int64; e: Float64; }
fn sum_ints(m: Mixed) -> Int {
  return m.a as Int + m.b as Int + m.c as Int + m.d as Int;
}
fn main() -> Int {
  var m = Mixed{ a: 10; b: 20; c: 30; d: 40; e: 3.5; };
  var total: Int = sum_ints(m);
  m.a = 5;
  m.e = 7.0;
  if total == 100 && m.a == 5 && m.e == 7.0 { return 0; }
  return 1;
}
