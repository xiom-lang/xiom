// G-15: sret ABI -- C struct return on Linux SysV
// Verifies XIOM correctly handles extern C functions returning structs by value.

type Small = { x: Int; y: Int; }
type Large = { a: Int; b: Int; c: Int; d: Int; e: Int; }

extern "C" {
  fn g15_small_return() -> Small;
  fn g15_large_return() -> Large;
  fn g15_pass_and_return(s: Small) -> Small;
}

fn main() -> Int {
  // D2.1 (Unsafe Confinement): extern "C" calls are confined to unsafe blocks.
  var small = unsafe { g15_small_return() };
  var large = unsafe { g15_large_return() };
  var rt = unsafe { g15_pass_and_return(small) };
  return 0;
}
