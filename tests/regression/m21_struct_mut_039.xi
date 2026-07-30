module m21_struct_mut_039
type Record = { a: Int; b: Int; }
fn main() -> Int {
  var r: Record = Record{ a: 5; b: 10; };
  var x = consume_a(r);
  if x == 5 { return 0; }
  return 1;
  return 1;
}
