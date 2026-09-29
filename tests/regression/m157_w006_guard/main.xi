// m157 (Stage 6 W006) negative: in-range literal shifts (Int, Int8, UInt8)
// and variable shift amounts must stay silent.
module m157_w006_guard;

fn main() -> Int {
  let x = 1;
  let n = 4;
  let a = x << 63;
  let b = x >> 0;
  let c = x << n;
  let small: Int8 = 3;
  let d = small << 7;
  let u: UInt8 = 3;
  let e = u << 7;
  let f = x << 3;
  if b != 1 { return 1; }
  if c != 16 { return 2; }
  if f != 8 { return 3; }
  return 0;
}
