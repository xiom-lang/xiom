// m391 (Stage 6 tier 3 W010) negative: `_`-prefixed parameters and used
// parameters must stay silent. Method parameters are skipped structurally.
module m391_w010_guard;

type Counter = { n: Int; }

fn keep(a: Int, _ignore: Int) -> Int {
  return a;
}

fn Counter.bump(self, amount: Int) -> Int {
  return self.n;
}

fn main() -> Int {
  let c = Counter { n: 3 };
  if keep(1, 2) != 1 { return 1; }
  if c.bump(9) != 3 { return 2; }
  return 0;
}
