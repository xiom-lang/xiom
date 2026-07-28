// M34-H07: Float64->Float32 narrowing — double to single precision
fn main() -> Int {
  var f64: Float64 = 3.141592653589793;
  var f32: Float32 = f64 as Float32;
  if f32 > 3.14 && f32 < 3.142 { return 0; }
  return 1;
}
