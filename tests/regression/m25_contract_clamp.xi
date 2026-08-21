// M25: Clamp with ensures -- result must be within range
fn clamp(x: Int, lo: Int, hi: Int) -> Int
  ensures: result >= lo
  ensures: result <= hi
{
  if x < lo { return lo; }
  if x > hi { return hi; }
  return x;
}
fn main() -> Int {
  var r1: Int = clamp(5, 0, 10);
  var r2: Int = clamp(-5, 0, 10);
  var r3: Int = clamp(15, 0, 10);
  if r1 == 5 && r2 == 0 && r3 == 10 { return 0; }
  return 1;
}
