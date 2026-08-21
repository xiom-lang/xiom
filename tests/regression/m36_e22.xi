// M36-E22: Arithmetic at type boundaries -- Int min / Int max
fn main() -> Int {
  var max_val = 2147483647; var min_val = -2147483648;
  var near_max = max_val - 1;
  var near_min = min_val + 1;
  if max_val <= 0 { return 1; }
  if min_val >= 0 { return 2; }
  if near_max != max_val - 1 { return 3; }
  if near_min != min_val + 1 { return 4; }
  var zero_plus = min_val + max_val;
  if zero_plus != -1 { return 5; }
  return 0;
}
