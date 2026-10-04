// Phase 3 pattern gate, accept case: m181 enum-declared Some/None variants
// belong to the enum, not Option.
enum Opt { None, Some }

fn f(o: Opt) -> Int {
  match o {
    Some => { return 1; }
    None => { return 0; }
  }
}
