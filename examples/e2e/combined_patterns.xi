// E2E: combined features -- store_back + hash + generic monomorphization
// Returns 0 on success.
module e2e_combined
use xiom.hash;

type Counter = { value: Int; }

fn Counter.inc(self) -> Counter {
  self.value = self.value + 1;
  self
}

fn main() -> Int {
  var c = Counter { value: 41; };
  c.inc(); // store_back should update c
  if c.value != 42 { return 1; }
  // Hash determinism
  let h1 = hash.hash(c.value);
  let h2 = hash.hash(c.value);
  if h1 != h2 || h1 == 0 { return 1; }
  return 0;
}
