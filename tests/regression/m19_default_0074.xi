module regression.m19_default_0074

interface Math {
  fn x(&self) -> Int { return a() + b(); }
  fn y(&self) -> Int { return b() + c(); }
  fn z(&self) -> Int { return a() + b() + c(); }
  fn a(&self) -> Int;
  fn b(&self) -> Int;
  fn c(&self) -> Int;
}

type Triple = { v1: Int; v2: Int; v3: Int; }

fn Triple.a(&self) -> Int { return v1; }

fn Triple.b(&self) -> Int { return v2; }

fn Triple.c(&self) -> Int { return v3; }

fn main() -> Int {
  var t: Triple = Triple{ v1: 1, v2: 2, v3: 3 };
  if t.a() == 1 && t.b() == 2 && t.c() == 3 && t.x() == 3 && t.y() == 5 && t.z() == 6 { return 0; }
  return 1;
}
