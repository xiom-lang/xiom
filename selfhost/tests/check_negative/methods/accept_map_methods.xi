// Phase 3 method gate, accept case: Map methods register from the
// collections module (interface/impl-style receiver fns, module-private).
use xiom.collections;

fn main() -> Int {
  var m = Map[Str, Int].new();
  m.insert("a", 1);
  match m.get("a") {
    Some(v) => { return v; }
    None => { return 0; }
  }
}
