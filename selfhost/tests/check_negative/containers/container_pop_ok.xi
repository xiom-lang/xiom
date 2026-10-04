// Phase 3 container gate, accept case: `Vec.pop` returns Option so match
// arm payloads bind.
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  match v.pop() {
    Some(x) => { return x; }
    None => { return 0; }
  }
}
