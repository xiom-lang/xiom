// M34-H06: Int->Float64 promotion — integer widened to float
fn main() -> Int {
  var i: Int = 42;
  var f: Float64 = i as Float64;
  if f > 41.9 && f < 42.1 { return 0; }
  return 1;
}
