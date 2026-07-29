module regression.m19_default_0069

interface Counter {
  fn next_count(&self) -> Int { return count() + step(); }
  fn count(&self) -> Int;
  fn step(&self) -> Int;
}

type Accumulator = { c: Int; inc: Int; }

fn Accumulator.count(&self) -> Int { return c; }

fn Accumulator.step(&self) -> Int { return inc; }

fn main() -> Int {
  var a: Accumulator = Accumulator{ c: 10, inc: 3 };
  if a.count() == 10 && a.step() == 3 && a.next_count() == 13 { return 0; }
  return 1;
}
