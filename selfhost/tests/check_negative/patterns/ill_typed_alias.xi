// Phase 3 pattern gate: m181 alias unwrapping -- an alias to Int still
// rejects `Some` and the message prints the ALIAS spelling.
type MyOpt = Int;

fn f(m: MyOpt) -> Int {
  match m {
    Some(x) => { return x; }
    _ => { return 0; }
  }
}
