// M34-J07: use module.Type — import specific types from modules
module data {
  pub type Record = { id: Int; name: Str; score: Float64; }
  pub fn make(id: Int, n: Str, s: Float64) -> Record {
    return Record{ id: id; name: n; score: s; };
  }
}
use data.Record;
use data.make;
fn sum_fields(r: Record) -> Int { return r.id + (r.name.len() as Int); }
fn main() -> Int {
  var r: Record = make(42, "test", 95.5);
  if r.id == 42 && r.score > 90.0 && sum_fields(r) == 46 { return 0; }
  return 1;
}
