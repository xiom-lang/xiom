// M32-G13: Multiple generic params via interface composition
interface Show { fn show(self) -> Int; }
interface Count { fn count(self) -> Int; }
type Record = { a: Int; b: Int; }
impl Show for Record {
  fn show(self) -> Int { return self.a; }
}
impl Count for Record {
  fn count(self) -> Int { return self.b; }
}
fn main() -> Int {
  var r = Record{ a: 5; b: 15 };
  var sa = r.show();
  var sb = r.count();
  if sa != 5 { return 1; }
  if sb != 15 { return 2; }
  if sa + sb != 20 { return 3; }
  return 0;
}
