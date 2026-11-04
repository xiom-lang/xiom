// m391 (Stage 6 tier 3 W010): an unused function parameter warns only under
// --lint-strict. Warning-only; the compile exits 0.
module m391_w010_unused_param;

fn add(a: Int, b: Int) -> Int {
  return a;
}

fn main() -> Int {
  if add(1, 2) != 1 { return 1; }
  return 0;
}
