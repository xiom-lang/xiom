// M35-T24: Array exhaustive — multi-dimensional, nested access
fn sum_int(arr: Vec[Int], n: Int) -> Int { var i: Int = 0; var s: Int = 0; while i < n { s = s + arr[i]; i = i + 1; } return s; }
fn first_char(arr: Vec[Char]) -> Char { return arr[0]; }
fn array_if(arr: Vec[Int]) -> Int { if arr[0] > 0 { return arr[0]; } return -1; }
fn main() -> Int {
  var ia: Vec[Int] = [1, 2, 3, 4, 5];
  if sum_int(ia, 5) != 15 { return 1; }
  var ca: Vec[Char] = ['X', 'Y', 'Z'];
  if first_char(ca) != 'X' { return 2; }
  var row1: Vec[Int] = [10, 20];
  if row1[0] + row1[1] != 30 { return 3; }
  if array_if(ia) != 1 { return 4; }
  return 0;
}

