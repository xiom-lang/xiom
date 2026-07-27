interface Ident { fn id(self) -> Int; }
type W = { n: Int; }
impl Ident for W {
  fn id(self) -> Int { return self.n; }
}
fn main() -> Int {
  var w = W{ n: 99 };
  if w.id() != 99 { return 1; }
  return 0;
}