// M32: Int16 comparisons at boundaries
fn main() -> Int {
  var min: Int16 = -32768 as Int16;
  var max: Int16 = 32767 as Int16;
  var zero: Int16 = 0;
  if min <= zero && zero <= max && min != max { return 0; }
  return 1;
}
