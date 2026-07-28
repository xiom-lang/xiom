// M36-E04: Empty enum — minimal enum with no payload variants (unit variants only)
enum VoidE { None }
fn main() -> Int {
  var v = VoidE.None;
  var ok = 0;
  match v {
    None => { ok = 1; }
  }
  if ok != 1 { return 1; }
  return 0;
}
