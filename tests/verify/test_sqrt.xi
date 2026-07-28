module test_sqrt
fn my_sqrt(x: Float64) -> Float64
  requires: x >= 0.0
  ensures: result >= 0.0
  ensures: result * result <= x + 0.000001
{ return x; }
