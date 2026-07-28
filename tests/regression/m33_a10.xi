// M33-A10: Array return — function returns a computed array element
fn compute_result() -> Int {
  return 42;
}
fn main() -> Int {
  var arr = [compute_result(), 100, 200];
  if arr[0] == 42 && arr[1] == 100 && arr[2] == 200 { return 0; }
  return 1;
}
