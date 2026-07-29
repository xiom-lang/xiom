module regression.m19_default_0125

interface Identifiable {
  fn id(&self) -> Str { return name() + "#" + key().to_str(); }
  fn name(&self) -> Str;
  fn key(&self) -> Int { return 0; }
}

interface Greeter {
  fn greet(&self) -> Str { return "Hi " + name(); }
  fn name(&self) -> Str;
}

interface Describable {
  fn desc(&self) -> Str { return "[" + id() + "]: " + greet(); }
}

type Person = { tag: Str; }

fn Person.name(&self) -> Str { return tag; }
fn Person.key(&self) -> Int { return 42; }

fn main() -> Int {
  var p: Person = Person{ tag: "name" };
  if p.name() != "name" { return 1; }
  if p.key() != 42 { return 2; }
  if p.id() != "name#42" { return 3; }
  if p.greet() != "Hi name" { return 4; }
  if p.desc() != "[name#42]: Hi name" { return 5; }
  return 0;
}
