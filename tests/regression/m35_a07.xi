// M35-A07: Partition counting — count elements < pivot and >= pivot
fn main() -> Int {
  var arr = [8, 3, 9, 1, 5, 7, 2, 6, 4];
  var n: Int = 9;
  var pivot: Int = 5;
  var less: Int = 0;
  var geq: Int = 0;
  var k: Int = 0;
  while k < n { if arr[k] < pivot { less = less + 1; } else { geq = geq + 1; } k = k + 1; }
  if less + geq != n { return 1; }
  if less != 4 { return 2; }
  if geq != 5 { return 3; }
  return 0;
}
