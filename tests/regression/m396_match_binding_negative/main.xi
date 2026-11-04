// m396 lock: match/pattern bindings are immutable (the pattern grammar has
// no explicit `var` form).
fn main() -> Int {
  let o = Some(1);
  match o {
    Some(v) => { v = 2; return v; }
    None => { return 0; }
  }
}
