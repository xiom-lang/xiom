interface Math { fn add(self, x: Int) -> Int; fn mul(self, x: Int) -> Int; }
type C = { v: Int; }
impl Math for C {
  fn add(self, x: Int) -> Int { return self.v + x; }
  fn mul(self, x: Int) -> Int { return self.v * x; }
}
fn main() -> Int {
  var c = C{ v: 10 };
  if c.add(5) != 15 { return 1; }
  if c.mul(3) != 30 { return 2; }
  return 0;
}