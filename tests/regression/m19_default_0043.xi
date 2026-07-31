module regression.m19_default_0043

interface Chained {
  fn triple(&self) -> Int { return value() * 3; }
  fn double_triple(&self) -> Int { return triple() * 2; }
  fn hex(&self) -> Int { return double_triple(); }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.triple(self) -> Int { return self.value() * 3; }

fn Num.double_triple(self) -> Int { return self.triple() * 2; }

fn Num.hex(self) -> Int { return self.double_triple(); }


fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 4 };
  if n.value() == 4 && n.triple() == 12 && n.double_triple() == 24 && n.hex() == 24 { return 0; }
  return 1;
}
