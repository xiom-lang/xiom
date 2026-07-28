// M34-V07: Float division safety (division by 1.0, self-division, small divisor)
fn main() -> Int {
  var a: Float64 = 42.0;
  var div_one: Float64 = a / 1.0;
  var div_self: Float64 = a / a;
  var half: Float64 = a / 2.0;
  if div_one == a && div_self == 1.0 && half == 21.0 {
    return 0;
  }
  return 1;
}
