// m396 positive lock: `let x: T;` is init-once -- the FIRST whole-binding
// assignment is the initialization; after that the binding is immutable.
fn main() -> Int {
  let x: Int;
  x = 5;
  let s: Str;
  s = "ok";
  if s != "ok" { return 1; }
  return x;
}
