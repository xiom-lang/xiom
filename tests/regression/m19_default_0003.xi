module regression.m19_default_0003

interface Describable {
  fn describe(&self) -> Str { return "default"; }
  fn name(&self) -> Str;
}

type Widget = { label: Str; }

fn Widget.name(&self) -> Str { return label; }

fn Widget.describe(&self) -> Str { return "custom"; }

fn main() -> Int {
  var w: Widget = Widget{ label: "btn" };
  if w.name() == "btn" && w.describe() == "custom" { return 0; }
  return 1;
}
