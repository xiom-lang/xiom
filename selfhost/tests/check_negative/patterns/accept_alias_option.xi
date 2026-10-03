// Phase 3 pattern gate, accept case: m181 alias unwrapping -- `type MyOpt =
// Option[Int]` accepts Some/None.
type MyOpt = Option[Int];

fn f(m: MyOpt) -> Int {
  match m {
    Some(x) => { return x; }
    None => { return 0; }
  }
}
