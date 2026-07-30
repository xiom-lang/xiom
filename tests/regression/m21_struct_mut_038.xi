module m21_struct_mut_038
type Multi = { a: Int; b: Int; c: Int; }
fn main() -> Int {
  var m: Multi = Multi{ a: 1; b: 2; c: 3; };
  m.a = 10;
  m.b = 20;
  m.c = 30;
  if m.a == 10 && m.b == 20 && m.c == 30 { return 0; }
  return 1;
  return 1;
}
