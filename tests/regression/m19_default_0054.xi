module regression.m19_default_0054

interface Defaults {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return 2; }
}

type Empty = {}

fn main() -> Int {
  var e: Empty = Empty{};
  if e.a() == 1 && e.b() == 2 { return 0; }
  return 1;
}
