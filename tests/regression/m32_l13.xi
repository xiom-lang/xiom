// M32-L13: Nested break -- break from inner loop, check condition in outer
fn main() -> Int {
  var i: Int = 1;
  var found: Int = 0;
  var result: Int = 0;
  while i <= 10 {
    var j: Int = 1;
    while j <= 10 {
      if i * j == 24 {
        found = 1;
        result = i;
        break;
      }
      j += 1;
    }
    if found == 1 { break; }
    i += 1;
  }
  // first i with i*j=24: i=3, j=8
  if found == 1 && result == 3 { return 0; }
  return 1;
}
