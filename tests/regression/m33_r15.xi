enum InnerOption { Has(val: Int), Empty }
enum OuterOption { SomeInner(content: InnerOption), NothingOuter }
fn main() -> Int {
  var v = OuterOption.SomeInner(InnerOption.Has(42));
  match v {
    SomeInner(inner) => { match inner { Has(n) => { if n != 42 { return 1; } } Empty => { return 2; } } }
    NothingOuter => { return 3; }
  }
  var w = OuterOption.SomeInner(InnerOption.Empty);
  match w {
    SomeInner(inner) => { match inner { Has(_) => { return 4; } Empty => {} } }
    NothingOuter => { return 5; }
  }
  var x = OuterOption.NothingOuter;
  match x {
    SomeInner(_) => { return 6; }
    NothingOuter => {}
  }
  return 0;
}
