module regression.m19_default_0007

interface Friendly {
  fn greet(&self) -> Str { return "Hello, " + name(); }
  fn name(&self) -> Str;
}

type User = { nick: Str; }

fn User.greet(self) -> Str { return "Hello, " + self.name(); }


fn User.name(&self) -> Str { return nick; }

fn main() -> Int {
  var u: User = User{ nick: "Bob" };
  if u.name() == "Bob" && u.greet() == "Hello, Bob" { return 0; }
  return 1;
}
