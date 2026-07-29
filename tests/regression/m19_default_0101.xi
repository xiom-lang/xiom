module regression.m19_default_0101

interface A {
  fn a(&self) -> Int { return 1; }
  fn name(&self) -> Str;
}

interface B {
  fn b(&self) -> Int { return 2; }
  fn name(&self) -> Str;
}

type Foo = { tag: Str; }

fn Foo.name(&self) -> Str { return tag; }

fn main() -> Int {
  var f: Foo = Foo{ tag: "test" };
  if f.name() != "test" { return 1; }
  if f.a() != 1 { return 2; }
  if f.b() != 2 { return 3; }
  return 0;
}
