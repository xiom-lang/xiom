module regression.m19_default_0066

interface Named {
  fn formal(&self) -> Str { return "Mr. " + name(); }
  fn name(&self) -> Str;
}

interface Described {
  fn description(&self) -> Str { return "[" + label() + "]"; }
  fn label(&self) -> Str;
}

type Entity = { n: Str; }

fn Entity.name(&self) -> Str { return n; }

fn Entity.label(&self) -> Str { return n; }

fn main() -> Int {
  var e: Entity = Entity{ n: "X" };
  if e.name() == "X" && e.formal() == "Mr. X" && e.label() == "X" && e.description() == "[X]" { return 0; }
  return 1;
}
