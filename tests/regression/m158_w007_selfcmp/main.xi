// m158 (Stage 6 W007): self-comparison on non-float types is always
// true/false and must warn (Int, Str, Bool).
module m158_w007_selfcmp;

fn main() -> Int {
  let x = 5;
  let s = "abc";
  let b = true;
  let t = x == x;
  let u = x != x;
  let v = s == s;
  let w = b != b;
  if !t { return 1; }
  if u { return 2; }
  if !v { return 3; }
  if w { return 4; }
  return 0;
}
