// M22: Integer type casts — upcast, downcast, signed/unsigned
fn main() -> Int {
  var a: Int8 = 42;
  var b: Int32 = a as Int32;
  var c: Int64 = b as Int64;
  var d: Int32 = c as Int32; // truncate back
  var e: Int8 = d as Int8;   // truncate to original
  if e == 42 as Int8 && b == 42 as Int32 && c == 42 as Int64 {
    return 0;
  }
  return 1;
}
