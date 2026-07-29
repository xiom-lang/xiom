module regression.m19_default_0073

interface MaxCheck {
  fn is_max(&self) -> Bool { return value() == 9223372036854775807; }
  fn value(&self) -> Int;
}

type Number = { val: Int; }

fn Number.value(&self) -> Int { return val; }

fn main() -> Int {
  var small: Number = Number{ val: 100 };
  var big: Number = Number{ val: 9223372036854775807 };
  if !small.is_max() && big.is_max() { return 0; }
  return 1;
}
