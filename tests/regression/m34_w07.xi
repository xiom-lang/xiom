// M34-W07: Shift by 0 — identity on multiple types
fn main() -> Int {
  var a: Int = 999;
  var b: Int32 = 999 as Int32;
  var c: UInt = 999;
  var sl: Int = a << 0;
  var sr: Int = a >> 0;
  var sl32: Int32 = b << 0;
  var sr32: Int32 = b >> 0;
  var ul: UInt = c << 0;
  var ur: UInt = c >> 0;
  if sl == 999 && sr == 999 && sl32 == 999 as Int32 && sr32 == 999 as Int32 && ul == 999 && ur == 999 { return 0; }
  return 1;
}
