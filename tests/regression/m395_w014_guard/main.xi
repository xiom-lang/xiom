// m395 (Stage 6 tier 3 W014) negative: asymmetric/other-literal shapes and
// float operands must stay silent under --lint-strict.
module m395_w014_guard;

fn main() -> Int {
  let x = 7;
  let y = 2;
  let f = 1.5;
  let a = 0 + x;
  let b = 0 - x;
  let c = x * 0;
  let d = x / y;
  let e = x + 1;
  let _g = f + 0;
  let _h = f * 1;
  if a + b + c + d + e != -3 { return 1; }
  if _g + _h < 0.0 { return 2; }
  return 0;
}
