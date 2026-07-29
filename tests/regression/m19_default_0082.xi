module regression.m19_default_0082

interface AllDefaults {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return 2; }
  fn c(&self) -> Int { return 3; }
}

type Empty = {}

fn main() -> Int {
  var e: Empty = Empty{};
  if e.a() == 1 && e.b() == 2 && e.c() == 3 { return 0; }
  return 1;
}
