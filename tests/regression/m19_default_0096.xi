module regression.m19_default_0096

interface RangeCheck {
  fn in_range(&self) -> Bool {
    var v = value();
    return v > min() && v < max() || v == -1;
  }
  fn value(&self) -> Int;
  fn min(&self) -> Int;
  fn max(&self) -> Int;
}

type Range = { v: Int; lo: Int; hi: Int; }

fn Range.in_range(self) -> Bool {
    var v = self.value();
    return v > self.min() && v < self.max() || v == -1;
  }


fn Range.value(&self) -> Int { return v; }

fn Range.min(&self) -> Int { return lo; }

fn Range.max(&self) -> Int { return hi; }

fn main() -> Int {
  var inside: Range = Range{ v: 5, lo: 0, hi: 10 };
  var outside: Range = Range{ v: 20, lo: 0, hi: 10 };
  var special: Range = Range{ v: -1, lo: 0, hi: 10 };
  if !inside.in_range() { return 1; }
  if outside.in_range() { return 2; }
  if !special.in_range() { return 3; }
  return 0;
}
