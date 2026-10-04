// Phase 3 lint gate: W004 duplicate payload variant.
enum Shape { Circle(r: Float64), Square(s: Float64) }

fn f(sh: Shape) -> Float64 {
  match sh {
    Shape.Circle(r) => { return r; }
    Shape.Circle(q) => { return q; }
    Shape.Square(s) => { return s; }
  }
  return 0.0;
}
