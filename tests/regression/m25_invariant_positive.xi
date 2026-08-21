// M25: Invariant on struct -- Positive value
type Positive = { val: Int; invariant: val > 0; }
fn main() -> Int {
  var p = Positive{ val: 42; };
  if p.val > 0 { return 0; }
  return 1;
}
