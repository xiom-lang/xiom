// m396 lock: a field store whose ROOT is a `let` binding is rejected.
type Point = { x: Int; y: Int; }

fn main() -> Int {
  let p = Point{ x: 1; y: 2; };
  p.x = 3;
  return 0;
}
