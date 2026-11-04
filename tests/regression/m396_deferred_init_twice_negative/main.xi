// m396 negative lock: deferred init is ONCE only; the second assignment is
// the immutable-binding error.
fn main() -> Int {
  let x: Int;
  x = 1;
  x = 2;
  return x;
}
