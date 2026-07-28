// M35-L04: Struct with Bool fields — verify bool layout and bit pattern
type Flags = { a: Bool; b: Bool; c: Bool; d: Bool; }

fn main() -> Int {
  var f = Flags{ a: true; b: false; c: true; d: false; };
  if f.a != true { return 1; }
  if f.b != false { return 2; }
  if f.c != true { return 3; }
  if f.d != false { return 4; }
  if f.a && !f.b && f.c && !f.d { return 0; }
  return 5;
}
