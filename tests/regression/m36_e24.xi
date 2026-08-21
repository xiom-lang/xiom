// M36-E24: Mixed type operations -- Int and Float64 interplay
fn main() -> Int {
  var i: Int = 10;
  var f: Float64 = 3.5;
  var sum = i + 1;
  var g: Float64 = 2.5;
  var prod: Float64 = f * g;
  if prod < 8.0 || prod > 9.0 { return 1; }
  var big: Int = 100;
  var small: Float64 = 0.001;
  if small <= 0.0 { return 2; }
  if big <= 0 { return 3; }
  return 0;
}
