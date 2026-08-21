// M34-N2-14: Contract with multi-contract nesting -- requires/ensures chain
fn scale_and_clamp(x: Int, lo: Int, hi: Int, factor: Int) -> Int
  requires: lo <= hi
  requires: factor > 0
  requires: x > 0
  ensures: result >= lo
  ensures: result <= hi * factor
{
  var scaled = x * factor;
  if scaled < lo { return lo; }
  if scaled > hi * factor { return hi * factor; }
  return scaled;
}
fn main() -> Int {
  var r1 = scale_and_clamp(3, 10, 50, 5);
  var r2 = scale_and_clamp(1, 10, 50, 5);
  var r3 = scale_and_clamp(20, 10, 50, 5);
  if r1 == 15 && r2 == 10 && r3 == 100 { return 0; }
  return 1;
}
