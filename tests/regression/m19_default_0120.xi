module regression.m19_default_0120

interface Arithmetic {
  fn x2(&self) -> Int { return val() * 2; }
  fn x3(&self) -> Int { return val() * 3; }
  fn sum(&self) -> Int { return val() + x2() + x3(); }
  fn val(&self) -> Int;
}

type Num = { n: Int; }

fn Num.x2(self) -> Int { return self.val() * 2; }

fn Num.x3(self) -> Int { return self.val() * 3; }

fn Num.sum(self) -> Int { return self.val() + self.x2() + self.x3(); }


fn Num.val(&self) -> Int { return n; }

fn main() -> Int {
  var x: Num = Num{ n: 5 };
  if x.val() != 5 { return 1; }
  if x.x2() != 10 { return 2; }
  if x.x3() != 15 { return 3; }
  if x.sum() != 30 { return 4; }
  return 0;
}
