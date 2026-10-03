// Phase 3 lint gate, accept case: the W000/W004/W006/W007 conditions that
// must stay silent -- catch-all Ident arms cover enums, Option[..] and Bool
// matches never exhaustiveness-warn, and float self-comparison is meaningful
// (NaN).
enum Shape { Circle(rad: Float64), Square(side: Float64) }

fn f(b: Bool, o: Option[Int], x: Float64, sh: Shape) -> Int {
  var r = 0;
  match b {
    true => { r = 1; }
  }
  match o {
    Some(v) => { r = v; }
  }
  match sh {
    other => { r = r + 1; }
  }
  if x == x { r = r + 1; }
  return r;
}
