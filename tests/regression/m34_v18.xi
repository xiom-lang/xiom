// M34-V18: Float while-loop accumulator
fn main() -> Int {
  var accum: Float64 = 0.0;
  var step: Float64 = 0.5;
  var i: Int = 0;
  while i < 10 {
    accum = accum + step;
    i = i + 1;
  }
  if accum == 5.0 {
    return 0;
  }
  return 1;
}
