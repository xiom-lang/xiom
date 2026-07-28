// M34-W11: Power-of-2 check — (x & (x - 1)) == 0
fn is_pow2(x: Int) -> Int {
  if x <= 0 { return 0; }
  if (x & (x - 1)) == 0 { return 1; }
  return 0;
}
fn main() -> Int {
  if is_pow2(0) != 0 { return 1; }
  if is_pow2(1) != 1 { return 2; }
  if is_pow2(2) != 1 { return 3; }
  if is_pow2(3) != 0 { return 4; }
  if is_pow2(4) != 1 { return 5; }
  if is_pow2(16) != 1 { return 6; }
  if is_pow2(1024) != 1 { return 7; }
  if is_pow2(100) != 0 { return 8; }
  if is_pow2(255) != 0 { return 9; }
  return 0;
}
