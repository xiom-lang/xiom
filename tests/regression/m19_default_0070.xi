module regression.m19_default_0070

interface Greeter {
  fn greet(&self) -> Str { return "hello"; }
}

type Dog = {}

fn Dog.greet(&self) -> Str { return "woof"; }

type Cat = {}

fn main() -> Int {
  var d: Dog = Dog{};
  var c: Cat = Cat{};
  if d.greet() == "woof" && c.greet() == "hello" { return 0; }
  return 1;
}
