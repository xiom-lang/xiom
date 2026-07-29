module regression.m19_default_0048

interface Rangable {
  fn in_range(&self) -> Bool {
    var v = value();
    return v > 0 && v < 100 || v == -1;
  }
  fn value(&self) -> Int;
}

type Check = { val: Int; }

fn Check.value(&self) -> Int { return val; }

fn main() -> Int {
  var a: Check = Check{ val: 50 };
  var b: Check = Check{ val: -1 };
  var c: Check = Check{ val: 150 };
  var d: Check = Check{ val: 0 };
  if !a.in_range() { return 1; }
  if !b.in_range() { return 2; }
  if c.in_range() { return 3; }
  if d.in_range() { return 4; }
  return 0;
}
