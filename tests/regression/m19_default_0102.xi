module regression.m19_default_0102

interface Greeter {
  fn greet(&self) -> Str { return "Hi " + name(); }
  fn name(&self) -> Str;
}

interface Printer {
  fn print(&self) -> Str { return "[" + label() + "]"; }
  fn label(&self) -> Str;
}

interface Mover {
  fn move(&self) -> Str { return "-> " + pos(); }
  fn pos(&self) -> Str;
}

type Entity = { tag: Str; }

fn Entity.name(&self) -> Str { return tag; }
fn Entity.label(&self) -> Str { return tag; }
fn Entity.pos(&self) -> Str { return tag; }

fn main() -> Int {
  var e: Entity = Entity{ tag: "X" };
  if e.name() != "X" { return 1; }
  if e.greet() != "Hi X" { return 2; }
  if e.label() != "X" { return 3; }
  if e.print() != "[X]" { return 4; }
  if e.pos() != "X" { return 5; }
  if e.move() != "-> X" { return 6; }
  return 0;
}
