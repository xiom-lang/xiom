// M32: Int32 arithmetic with elif/else magnitude check
fn main() -> Int {
  var a: Int32 = 1000000;
  var b: Int32 = -500000;
  var sum: Int32 = a + b;
  var prod: Int32 = a * 2;
  if sum == 500000 as Int32 {
    if prod == 2000000 as Int32 {
      return 0;
    } elif prod > 1000000 as Int32 {
      return 1;
    }
    return 2;
  } elif sum < 0 as Int32 {
    return 3;
  } else {
    return 4;
  }
  return 5;
}
