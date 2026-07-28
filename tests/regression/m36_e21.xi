// M36-E21: Chained method calls
type C = { val: Int; }
fn C.inc(self) -> C { return C{ val: self.val + 1 }; }
fn C.double(self) -> C { return C{ val: self.val * 2 }; }
fn C.add(self, n: Int) -> C { return C{ val: self.val + n }; }
fn main() -> Int {
  var c = C{ val: 0 };
  var r = c.inc().double().add(10).inc();
  if r.val != 13 { return 1; }
  return 0;
}