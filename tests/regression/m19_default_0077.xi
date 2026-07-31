module regression.m19_default_0077

interface DeepChain {
  fn x2(&self) -> Int { return val() * 2; }
  fn x4(&self) -> Int { return x2() * 2; }
  fn x8(&self) -> Int { return x4() * 2; }
  fn val(&self) -> Int;
}

type Num = { x: Int; }

fn Num.x2(self) -> Int { return self.val() * 2; }

fn Num.x4(self) -> Int { return self.x2() * 2; }

fn Num.x8(self) -> Int { return self.x4() * 2; }


fn Num.val(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 3 };
  if n.val() == 3 && n.x2() == 6 && n.x4() == 12 && n.x8() == 24 { return 0; }
  return 1;
}
