// m393 (Stage 6 tier 3 W012): literal `if true/false`, literal `elif` and
// `while false` conditions warn only under --lint-strict. `while true` is
// deliberately NOT reported. Warning-only; the compile exits 0.
module m393_w012_constant_cond;

fn main() -> Int {
  var n = 0;
  if true { n = 1; } else { n = 2; }
  if false { n = 3; } elif true { n = 4; } else { n = 5; }
  while false { n = n + 1; }
  if n != 4 { return 1; }
  return 0;
}
