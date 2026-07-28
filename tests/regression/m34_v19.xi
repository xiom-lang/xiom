// M34-V19: Float array operations (literal, index access, sum)
fn main() -> Int {
  var arr = [1.0, 2.0, 3.0, 4.0, 5.0];
  var sum: Float64 = 0.0;
  var i: Int = 0;
  while i < 5 {
    sum = sum + arr[i];
    i = i + 1;
  }
  var prod: Float64 = arr[0] * arr[4];
  if sum == 15.0 && prod == 5.0 {
    return 0;
  }
  return 1;
}
