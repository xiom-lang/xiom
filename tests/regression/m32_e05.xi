// M32-E05: Nested match
enum A { X, Y(val: Int) }
enum B { P, Q(inner: A) }
fn main() -> Int {
  var b = B.Q(A.Y(7));
  match b {
    P => { return 1; }
    Q(inner) => {
      match inner {
        X => { return 2; }
        Y(v) => { if v != 7 { return 3; } }
      }
    }
  }
  return 0;
}
