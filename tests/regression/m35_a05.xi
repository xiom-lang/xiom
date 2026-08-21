// M35-A05: Insertion position -- find where a value belongs in sorted array
fn main() -> Int {
  var arr = [5, 6, 11, 12, 15];
  var n: Int = 5;
  var val: Int = 7;
  var pos: Int = 0;
  while pos < n && arr[pos] < val { pos = pos + 1; }
  if pos != 2 { return 1; }
  val = 1; pos = 0;
  while pos < n && arr[pos] < val { pos = pos + 1; }
  if pos != 0 { return 2; }
  val = 20; pos = 0;
  while pos < n && arr[pos] < val { pos = pos + 1; }
  if pos != 5 { return 3; }
  return 0;
}
