// m396 negative lock: `let x: T = 0;` is initialized AT DECLARATION, so the
// first later store is already an error (the parser placeholder must not be
// confused with an explicit literal zero).
fn main() -> Int {
  let x: Int = 0;
  x = 1;
  return x;
}
