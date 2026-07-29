module regression.m19_default_0009

interface Validatable {
  fn is_valid(&self) -> Bool { return value() > 0; }
  fn value(&self) -> Int;
}

type Metric = { val: Int; }

fn Metric.value(&self) -> Int { return val; }

fn main() -> Int {
  var pos: Metric = Metric{ val: 5 };
  var neg: Metric = Metric{ val: -3 };
  if pos.is_valid() && !neg.is_valid() { return 0; }
  return 1;
}
