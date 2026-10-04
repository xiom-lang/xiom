// Phase 3 method gate, accept case: interface-typed receivers dispatch to
// declared members.
interface Shape {
  fn area(self) -> Float64;
}

fn f(s: Shape) -> Float64 {
  return s.area();
}
