module regression.m19_default_0116

interface Greeter {
  fn greet(&self) -> Str { return "Hello " + name(); }
  fn name(&self) -> Str;
}

type Person = { tag: Str; }

fn Person.greet(self) -> Str { return "Hello " + self.name(); }


fn Person.name(&self) -> Str { return tag; }

type Dog = { tag: Str; }

fn Dog.name(&self) -> Str { return tag; }

type Cat = { tag: Str; }

fn Cat.name(&self) -> Str { return tag; }

fn main() -> Int {
  var p: Person = Person{ tag: "Alice" };
  var d: Dog = Dog{ tag: "Rex" };
  var c: Cat = Cat{ tag: "Whiskers" };
  if p.name() != "Alice" { return 1; }
  if p.greet() != "Hello Alice" { return 2; }
  if d.name() != "Rex" { return 3; }
  if d.greet() != "Hello Rex" { return 4; }
  if c.name() != "Whiskers" { return 5; }
  if c.greet() != "Hello Whiskers" { return 6; }
  return 0;
}
