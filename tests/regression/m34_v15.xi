// M34-V15: Float const declarations
const PI: Float64 = 3.14;
const E: Float64 = 2.718;
const ZERO_F: Float64 = 0.0;
const NEG_ONE: Float64 = -1.0;
fn main() -> Int {
  var circle: Float64 = 2.0 * PI * 5.0;
  if PI > 3.0 && E > 2.0 && ZERO_F == 0.0 && NEG_ONE < 0.0 {
    return 0;
  }
  return 1;
}
