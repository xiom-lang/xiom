// M33-A13: Very large array — sum of 100 elements 0..99
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  while i < 100 {
    sum += i;
    i += 1;
  }
  if sum == 4950 && i == 100 { return 0; }
  return 1;
}
