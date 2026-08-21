// M35-A27: Maximum subarray sum -- Kadane's algorithm on an integer array
fn main() -> Int {
  var arr = [-2, 1, -3, 4, -1, 2, 1, -5, 4];
  var n: Int = 9;
  var max_so_far: Int = arr[0];
  var max_ending: Int = arr[0];
  var i: Int = 1;
  while i < n {
    if max_ending + arr[i] > arr[i] { max_ending = max_ending + arr[i]; } else { max_ending = arr[i]; }
    if max_so_far < max_ending { max_so_far = max_ending; }
    i = i + 1;
  }
  if max_so_far == 6 { return 0; }
  return 1;
}
