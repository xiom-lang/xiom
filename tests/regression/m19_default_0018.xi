module regression.m19_default_0018

interface Taggable {
  fn describe(&self) -> Str { return "enum"; }
  fn kind(&self) -> Str;
}

enum Color {
  Red,
  Green,
  Blue
}

fn Color.kind(&self) -> Str { return "color"; }

fn main() -> Int {
  var c: Color = Color.Red;
  if c.kind() == "color" && c.describe() == "enum" { return 0; }
  return 1;
}
