// M32: Int8 comparisons at boundaries
fn main() -> Int {
  var min: Int8 = -128 as Int8;
  var max: Int8 = 127 as Int8;
  var zero: Int8 = 0;
  if min < max && min < zero && max > zero { return 0; }
  return 1;
}
