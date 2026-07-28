// M33-B13: Borrow in match — borrow enum, match owned enum to extract payload
type Opt = enum { Has(v: Int), Empty, }
fn classify(x: &Opt) -> Int {
  return 1;
}
fn main() -> Int {
  var o = Opt.Has(77);
  var kind = classify(&o);
  match o {
    Has(v) => {
      if v != 77 { return 1; }
    }
    Empty => {
      return 3;
    }
  }
  if kind != 1 { return 4; }
  return 0;
}
