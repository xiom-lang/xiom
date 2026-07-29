module regression.m19_default_0122

interface Constants {
  fn is_always(&self) -> Bool { return true; }
  fn is_never(&self) -> Bool { return false; }
  fn label(&self) -> Str;
}

type Item = { tag: Str; }

fn Item.label(&self) -> Str { return tag; }

fn main() -> Int {
  var i: Item = Item{ tag: "x" };
  if i.label() != "x" { return 1; }
  if i.is_always() != true { return 2; }
  if i.is_never() != false { return 3; }
  return 0;
}
