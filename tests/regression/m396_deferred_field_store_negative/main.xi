// m396 lock: a field store is NOT an initialization of a deferred `let`.
// Only the first bare `p = ...` may initialize it; `p.x = ...` before that
// is still an immutable/uninitialized-root error.
type Point = { x: Int; }

fn main() -> Int {
  let p: Point;
  p.x = 1;
  return 0;
}
