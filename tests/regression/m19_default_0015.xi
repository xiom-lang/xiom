module regression.m19_default_0015

interface Named {
  fn greet(&self) -> Str { return "Hello, " + name(); }
  fn name(&self) -> Str;
}

type Person = { tag: Str; }

fn Person.greet(self) -> Str { return "Hello, " + self.name(); }


fn Person.name(&self) -> Str { return tag; }

type Company = { brand: Str; }

fn Company.name(&self) -> Str { return brand; }

fn main() -> Int {
  var p: Person = Person{ tag: "Alice" };
  var c: Company = Company{ brand: "Acme" };
  if p.greet() == "Hello, Alice" && c.greet() == "Hello, Acme" { return 0; }
  return 1;
}
