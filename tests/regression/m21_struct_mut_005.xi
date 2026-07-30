module m21_struct_mut_005
type Record = { a: Int8; b: Int8; }
fn main() -> Int {
  var r: Record = Record{ a: 1 as Int8; b: 2 as Int8; };
  r.a = 100 as Int8;
  if r.a == 100 as Int8 && r.b == 2 as Int8 { return 0; }
  return 1;
}
