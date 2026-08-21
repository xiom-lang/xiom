// M32-L04: Nested while -- multiplication table sum (1..5 x 1..5)
fn main() -> Int {
  var i: Int = 1;
  var sum: Int = 0;
  while i <= 5 {
    var j: Int = 1;
    while j <= 5 {
      sum += i * j;
      j += 1;
    }
    i += 1;
  }
  // sum of all i*j for i,j in 1..5 = (1+2+3+4+5)2 = 152 = 225
  if sum == 225 { return 0; }
  return 1;
}
