// m156 (Stage 6 W008) negative: float division by zero (IEEE inf), an int
// zero literal adopting a float operand, and non-literal divisors must all
// stay silent.
module m156_w008_guard;

fn main() -> Int {
  let x = 10;
  let y = 2;
  let f = 1.5;
  let a = f / 0.0;
  let b = f / 0;
  let c = x / y;
  let d = x % 3;
  if a <= 0.0 { return 1; }
  if b <= 0.0 { return 2; }
  if c != 5 { return 3; }
  if d != 1 { return 4; }
  return 0;
}
