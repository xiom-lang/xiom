// M32: Struct with UInt8 field and cross-boundary field access
type Rec = { id: UInt8; score: UInt16; }
fn main() -> Int {
  var r: Rec = Rec{ id: 200; score: 50000 };
  var sum: Int = r.id as Int + r.score as Int;
  if sum == 50200 { return 0; }
  return 1;
}
