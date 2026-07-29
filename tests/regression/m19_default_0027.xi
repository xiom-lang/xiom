module regression.m19_default_0027

interface Incrementable {
  fn inc(&self) -> Int { return value() + 1; }
  fn value(&self) -> Int;
}

type Number = { n: Int; }

fn Number.value(&self) -> Int { return n; }

fn main() -> Int {
  var x: Number = Number{ n: 10 };
  if x.value() == 10 && x.inc() == 11 { return 0; }
  return 1;
}
