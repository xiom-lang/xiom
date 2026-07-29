module regression.m19_default_0080

interface A {
  fn tag(&self) -> Str { return "A"; }
}

interface B {
  fn tag(&self) -> Str { return "B"; }
}

type Item = {}

fn Item.tag(&self) -> Str { return "C"; }

fn main() -> Int {
  var i: Item = Item{};
  if i.tag() == "C" { return 0; }
  return 1;
}
