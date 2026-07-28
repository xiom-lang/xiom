// M35-A02: Linear search — inline sequential scan in main
fn main() -> Int {
  var arr = [42, 17, 99, 3, 71, 56];
  var n: Int = 6;
  var found: Int = -1;
  var i: Int = 0;
  while i < n { if arr[i] == 99 { found = i; } i = i + 1; }
  if found != 2 { return 1; }
  found = -1; i = 0;
  while i < n { if arr[i] == 42 { found = i; } i = i + 1; }
  if found != 0 { return 2; }
  found = -1; i = 0;
  while i < n { if arr[i] == 100 { found = i; } i = i + 1; }
  if found != -1 { return 3; }
  return 0;
}
