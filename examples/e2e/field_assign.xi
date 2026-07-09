// E2E: struct field assignment store-through
// Verifies that `self.field = expr` emits the store instruction
// (fixed in Clusters 1-2). Returns 0 on success.

module e2e_field_assign

type Counter = { value: Int; }

fn Counter.inc(self) -> Counter {
  self.value = self.value + 1;
  self
}

fn main() -> Int {
  let c = Counter { value: 41; };
  let c2 = c.inc();
  if c2.value == 42 {
    return 0;
  }
  return 1;
}
