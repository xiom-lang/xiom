// M32-L15: While loop with break on negative -- sum positives until negative
fn main() -> Int {
  var arr = [5, 12, 8, -3, 20, 15];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 6 {
    if arr[i] < 0 { break; }
    sum += arr[i];
    i += 1;
  }
  // sum: 5+12+8 = 25 (stops at -3)
  if sum == 25 { return 0; }
  return 1;
}
