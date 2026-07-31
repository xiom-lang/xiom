module regression.m19_default_0113

interface Accumulator {
  fn double(&self) -> Int { return get() * 2; }
  fn triple(&self) -> Int { return get() * 3; }
  fn get(&self) -> Int;
}

type Counter = { n: Int; }

fn Counter.double(self) -> Int { return self.get() * 2; }

fn Counter.triple(self) -> Int { return self.get() * 3; }


fn Counter.get(&self) -> Int { return n; }

fn main() -> Int {
  var c: Counter = Counter{ n: 7 };
  if c.get() != 7 { return 1; }
  if c.double() != 14 { return 2; }
  if c.triple() != 21 { return 3; }
  return 0;
}
