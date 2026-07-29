module regression.m19_default_0121

interface Bounded {
  fn is_valid(&self) -> Bool { var v = value(); return v >= 0 && v < limit(); }
  fn value(&self) -> Int;
  fn limit(&self) -> Int;
}

type Buf = { data: Int; max: Int; }

fn Buf.value(&self) -> Int { return data; }
fn Buf.limit(&self) -> Int { return max; }

fn main() -> Int {
  var b: Buf = Buf{ data: 42, max: 100 };
  if b.value() != 42 { return 1; }
  if b.limit() != 100 { return 2; }
  if b.is_valid() != true { return 3; }
  return 0;
}
