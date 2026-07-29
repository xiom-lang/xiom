module regression.m19_default_0040

interface Factorial {
  fn factorial(&self) -> Int {
    var n = value();
    var result = 1;
    while n > 1 {
      result = result * n;
      n = n - 1;
    }
    return result;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var f: Num = Num{ x: 5 };
  if f.value() == 5 && f.factorial() == 120 { return 0; }
  return 1;
}
