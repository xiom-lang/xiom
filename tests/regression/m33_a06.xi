// M33-A06: Array of Bool -- creation and explicit bool variable checks
fn main() -> Int {
  var arr = [true, false, true, false, true];
  var a: Bool = arr[0];
  var b: Bool = arr[1];
  var c: Bool = arr[2];
  var d: Bool = arr[3];
  var e: Bool = arr[4];
  if a && !b && c && !d && e { return 0; }
  return 1;
}
