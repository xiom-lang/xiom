// m395 (Stage 6 tier 3 W014): vacuous integer arithmetic (`x + 0`, `x - 0`,
// `x * 1`, `x / 1`) warns only under --lint-strict. Warning-only; the
// compile exits 0.
module m395_w014_vacuous_arith;

fn main() -> Int {
  let x = 7;
  let a = x + 0;
  let b = x - 0;
  let c = x * 1;
  let d = x / 1;
  if a != 7 || b != 7 || c != 7 || d != 7 { return 1; }
  return 0;
}
