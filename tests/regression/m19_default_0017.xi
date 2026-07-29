module regression.m19_default_0017

interface Greeter {
  fn greet(&self) -> Str { return "Hello"; }
  fn name(&self) -> Str;
}

interface Printer {
  fn print(&self) -> Str { return "[" + name() + "]"; }
  fn name(&self) -> Str;
}

type Entity = { id: Str; }

fn Entity.name(&self) -> Str { return id; }

fn main() -> Int {
  var e: Entity = Entity{ id: "X" };
  if e.name() == "X" && e.greet() == "Hello" && e.print() == "[X]" { return 0; }
  return 1;
}
