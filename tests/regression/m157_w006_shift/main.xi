// m157 (Stage 6 W006): a shift by a literal amount >= the left operand's
// bit width must warn -- type aware (Int is 64-bit, Int8 is 8-bit).
// Warning-only: compilation stays exit 0; the binary is NOT executed
// (`1 << 64` yields a garbage value at runtime -- the trap-less failure
// this lint exists for).
module m157_w006_shift;

fn wide(n: Int) -> Int {
  let a = 1 << 64;
  let b = n >> 64;
  return a + b;
}

fn narrow(n: Int8) -> Int8 {
  return n << 8;
}

fn main() -> Int {
  return wide(2);
}
