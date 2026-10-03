// Phase 3 lint gate: W000 non-exhaustive user enum (single missing variant;
// Rust's multi-missing order is HashMap-random, see COMPILER_BUGS).
enum Shape { Circle(r: Float64), Square(s: Float64) }

fn f(sh: Shape) -> Float64 {
  match sh {
    Shape.Circle(r) => { return r; }
  }
  return 0.0;
}
