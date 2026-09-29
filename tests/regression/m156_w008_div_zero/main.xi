// m156 (Stage 6 W008): a literal integer division/remainder by zero must
// warn. Warning-only: compilation stays exit 0; the binary is deliberately
// NOT executed -- both shapes trap at runtime (exit 0xC000001D).
module m156_w008_div_zero;

fn main() -> Int {
  let d = 10 / 0;
  let r = 7 % 0;
  return d + r;
}
