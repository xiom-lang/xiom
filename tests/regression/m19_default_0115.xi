module regression.m19_default_0115

interface Chain {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return a() * 2; }
  fn c(&self) -> Int { return b() * 3; }
  fn d(&self) -> Int { return c() * 4; }
  fn e(&self) -> Int { return d() * 5; }
}

type Node = {}

fn main() -> Int {
  var n: Node = Node{};
  if n.a() != 1 { return 1; }
  if n.b() != 2 { return 2; }
  if n.c() != 6 { return 3; }
  if n.d() != 24 { return 4; }
  if n.e() != 120 { return 5; }
  return 0;
}
