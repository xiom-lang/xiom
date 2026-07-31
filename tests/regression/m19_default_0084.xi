module regression.m19_default_0084

interface Applicable {
  fn apply(&self) -> Int {
    var f = |x| x * 2;
    return f(value());
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.apply(self) -> Int {
    var f = |x| x * 2;
    return f(self.value());
  }


fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 5 };
  if n.value() == 5 && n.apply() == 10 { return 0; }
  return 1;
}
