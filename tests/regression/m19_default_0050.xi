module regression.m19_default_0050

interface Steppable {
  fn next(&self) -> Int { return value() + 1; }
  fn value(&self) -> Int;
}

type Counter = { count: Int; }

fn Counter.next(self) -> Int { return self.value() + 1; }


fn Counter.value(&self) -> Int { return count; }

fn main() -> Int {
  var c: Counter = Counter{ count: 99 };
  if c.value() == 99 && c.next() == 100 { return 0; }
  return 1;
}
