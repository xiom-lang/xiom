// M34-H17: Cast in array index — index expression uses explicit cast
fn main() -> Int {
  var arr = [10, 20, 30, 40, 50];
  var idx: Int16 = 2;
  var val: Int = arr[idx as Int];
  if val == 30 { return 0; }
  return 1;
}
