// M36-X17: Method resolution — method calls on types with self methods
type Counter = { value: Int; }
fn Counter.new() -> Counter { return Counter{ value: 0; }; }
fn Counter.incr(self) -> Counter {
  var c = self;
  c.value = c.value + 1;
  return c;
}
fn Counter.add(self, n: Int) -> Counter {
  var c = self;
  c.value = c.value + n;
  return c;
}
fn Counter.decr(self) -> Counter {
  var c = self;
  c.value = c.value - 1;
  return c;
}
fn Counter.get(self) -> Int { return self.value; }
fn Counter.reset() -> Counter { return Counter{ value: 0; }; }
fn main() -> Int {
  var c1 = Counter.new();
  if c1.get() != 0 { return 1; }
  var c2 = c1.incr();
  if c2.get() != 1 { return 2; }
  var c3 = c2.incr().incr();
  if c3.get() != 3 { return 3; }
  var c4 = c3.add(5);
  if c4.get() != 8 { return 4; }
  var c5 = c4.decr();
  if c5.get() != 7 { return 5; }
  var c0 = Counter.reset();
  if c0.get() != 0 { return 6; }
  var chain = Counter.new().incr().incr().add(10).decr();
  if chain.get() != 11 { return 7; }
  return 0;
}
