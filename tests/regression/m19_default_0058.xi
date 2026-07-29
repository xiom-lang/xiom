module regression.m19_default_0058

interface Calculator {
  fn double(&self) -> Int { return a() * 2; }
  fn triple(&self) -> Int { return a() * 3; }
  fn a(&self) -> Int;
}

type Number = { val: Int; }

fn Number.a(&self) -> Int { return val; }

fn main() -> Int {
  var n: Number = Number{ val: 5 };
  if n.a() == 5 && n.double() == 10 && n.triple() == 15 { return 0; }
  return 1;
}
