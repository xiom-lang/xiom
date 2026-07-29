module regression.m19_default_0001

interface Greeter {
  fn greet(&self) -> Str { return "hello"; }
  fn name(&self) -> Str;
}

type Person = { tag: Str; }

fn Person.name(&self) -> Str { return tag; }

fn main() -> Int {
  var p: Person = Person{ tag: "alice" };
  if p.name() == "alice" && p.greet() == "hello" { return 0; }
  return 1;
}
