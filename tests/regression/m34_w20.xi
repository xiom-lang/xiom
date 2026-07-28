// M34-W20: Comprehensive bitwise stress — all operators on Int, Int32, UInt, UInt32
fn main() -> Int {
  // --- Int (i64) ---
  var i: Int = 0xAAAAAAAA;
  if (i & (-1 as Int)) != i { return 1; }
  if (i | (0 as Int)) != i { return 2; }
  if (i ^ i) != 0 { return 3; }
  if ~(~(i)) != i { return 4; }
  if (i << 1) != 0x155555554 { return 5; }
  if (i >> 1) != 0x55555555 { return 6; }
  // --- Int32 ---
  var i32: Int32 = 0x3333 as Int32;
  if (i32 & (0xFFFFFFFF as Int32)) != i32 { return 7; }
  if (i32 | (0 as Int32)) != i32 { return 8; }
  if (i32 ^ i32) != 0 as Int32 { return 9; }
  if (i32 << 4) != 0x33330 as Int32 { return 10; }
  if (i32 >> 4) != 0x333 as Int32 { return 11; }
  // --- Int64 ---
  var i64: Int64 = 0xFFFF as Int64;
  if (i64 & (0xFFFFFFFFFFFFFFFF as Int64)) != i64 { return 12; }
  if (i64 | (0 as Int64)) != i64 { return 13; }
  if (i64 ^ i64) != 0 as Int64 { return 14; }
  // --- UInt (u64) ---
  var u: UInt = 0xBBBBBBBB;
  if (u & (0xFFFFFFFFFFFFFFFF as UInt)) != u { return 15; }
  if (u | (0 as UInt)) != u { return 16; }
  if (u ^ u) != 0 { return 17; }
  if ~(~(u)) != u { return 18; }
  if (u << 2) != 0x2EEEEEEEC { return 19; }
  if (u >> 2) != 0x2EEEEEEE { return 20; }
  // --- UInt32 ---
  var u32: UInt32 = 0xCCCC as UInt32;
  if (u32 & (0xFFFFFFFF as UInt32)) != u32 { return 21; }
  if (u32 | (0 as UInt32)) != u32 { return 22; }
  if (u32 ^ u32) != 0 as UInt32 { return 23; }
  // --- Cross-type AND: Int & Int32 ---
  var si: Int32 = 0x5555 as Int32;
  var mix: Int32 = si & (0x7FFF as Int32);
  if mix != 0x5555 as Int32 { return 24; }
  return 0;
}
