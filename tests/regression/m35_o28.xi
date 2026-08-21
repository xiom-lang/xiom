// M35-O28: Option of Result -- Option and Result interaction via match
enum OptRes { Has(v: Int), Missing, Failed(msg: Str) }
fn safe_fetch(x: Int) -> OptRes {
  if x < 0 { return OptRes.Failed("negative"); }
  if x == 0 { return OptRes.Missing; }
  return OptRes.Has(x * 2);
}
fn main() -> Int {
  match safe_fetch(5) { OptRes.Has(v) => { if v != 10 { return 1; } } _ => { return 2; } }
  match safe_fetch(0) { OptRes.Missing => {} _ => { return 3; } }
  match safe_fetch(-1) { OptRes.Failed(msg) => { if msg != "negative" { return 4; } } _ => { return 5; } }
  return 0;
}
