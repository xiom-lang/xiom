module regression.m19_default_0045

interface Labelled {
  fn label(&self) -> Str { return "item"; }
  fn name(&self) -> Str;
}

type Apple = { variety: Str; }

fn Apple.label(self) -> Str { return "item"; }


fn Apple.name(&self) -> Str { return variety; }

type Orange = { variety: Str; }

fn Orange.name(&self) -> Str { return variety; }

fn Orange.label(&self) -> Str { return "fruit"; }

fn main() -> Int {
  var a: Apple = Apple{ variety: "gala" };
  var o: Orange = Orange{ variety: "navel" };
  if a.name() != "gala" { return 1; }
  if a.label() != "item" { return 2; }
  if o.name() != "navel" { return 3; }
  if o.label() != "fruit" { return 4; }
  return 0;
}
