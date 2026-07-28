// M32: Bitwise NOT on all integer types
fn main() -> Int {
  var a: Int8 = 0;
  var b: Int16 = 0;
  var c: Int32 = 0;
  var d: Int = 0;
  if ~a == -1 as Int8 && ~b == -1 as Int16 && ~c == -1 as Int32 && ~d == -1 {
    return 0;
  }
  return 1;
}
