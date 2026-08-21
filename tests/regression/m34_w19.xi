// M34-W19: Cross-type bitwise -- Int32 & UInt32 with explicit casts
fn main() -> Int {
  var si: Int32 = 0x7FFFFFFF as Int32;
  var ui: UInt32 = 0x80000000 as UInt32;
  // Int32 & UInt32 (cast UInt32 to Int32)
  var and_val: Int32 = si & (ui as Int32);
  // Int32 | UInt32 (cast Int32 to UInt32)
  var or_val: UInt32 = (si as UInt32) | ui;
  // Int32 ^ UInt32
  var xor_val: Int32 = si ^ (ui as Int32);
  // Verify: AND should be 0 since bits don't overlap
  if and_val != 0 as Int32 { return 1; }
  // Verify: OR should be all 1 bits of the lower 31 + MSB = 0xFFFFFFFF
  if or_val != 0xFFFFFFFF as UInt32 { return 2; }
  // Verify: XOR should be the same as OR when bits don't overlap
  if xor_val != 0xFFFFFFFF as Int32 { return 3; }
  return 0;
}
