// M35-O29: Result of Option -- Result and Option interaction via match
enum ResOpt { Found(v: Int), NotFound, Error(reason: Str) }
fn safe_lookup(idx: Int) -> ResOpt {
  if idx < 0 { return ResOpt.Error("bad index"); }
  if idx > 5 { return ResOpt.NotFound; }
  return ResOpt.Found(idx * 10);
}
fn main() -> Int {
  match safe_lookup(3) { ResOpt.Found(v) => { if v != 30 { return 1; } } _ => { return 2; } }
  match safe_lookup(10) { ResOpt.NotFound => {} _ => { return 3; } }
  match safe_lookup(-1) { ResOpt.Error(reason) => { if reason != "bad index" { return 4; } } _ => { return 5; } }
  return 0;
}
