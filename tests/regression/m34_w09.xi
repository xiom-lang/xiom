// M34-W09: Sign bit manipulation — MSB test on Int (avoid -1 as Int32 codegen bug)
fn main() -> Int {
  var pos: Int = 1;
  var neg: Int = -1;
  var msb_pos: Int = pos >> 63;
  var msb_neg: Int = neg >> 63;
  if msb_pos == 0 && msb_neg == -1 { return 0; }
  return 1;
}
