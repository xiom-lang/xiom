// M32-L02: While with break — accumulate until sum exceeds 50
fn main() -> Int {
  var i: Int = 1;
  var sum: Int = 0;
  while i < 100 {
    sum += i;
    if sum > 50 { break; }
    i += 1;
  }
  // i=1→1, 2→3, 3→6, 4→10, 5→15, 6→21, 7→28, 8→36, 9→45, 10→55 → break
  // sum = 55, i = 10
  if sum == 55 && i == 10 { return 0; }
  return 1;
}
