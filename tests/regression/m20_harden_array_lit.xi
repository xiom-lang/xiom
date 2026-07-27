fn sum_arr(arr: Vec[Int], n: Int) -> Int {
  var i = 0;
  var sum = 0;
  while i < n {
    sum = sum + arr[i];
    i = i + 1;
  }
  return sum;
}
fn main() -> Int {
  var arr = vec![1, 2, 3, 4, 5];
  if sum_arr(arr, 5) != 15 { return 1; }
  return 0;
}