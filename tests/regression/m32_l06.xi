// M32-L06: While with early return — find first square > 50
fn main() -> Int {
  var i: Int = 1;
  while i < 20 {
    var sq: Int = i * i;
    if sq > 50 { return 0; }
    i += 1;
  }
  return 1;
}
