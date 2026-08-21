// M33-A20: Array with contract -- function with requires/ensures on array length
fn triple(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
{
  return x * 3;
}
fn main() -> Int {
  var arr = [10, 20, 30, 40];
  var sum: Int = 0;
  sum += triple(arr[0]);
  sum += triple(arr[1]);
  sum += triple(arr[2]);
  sum += triple(arr[3]);
  if sum == 300 { return 0; }
  return 1;
}
