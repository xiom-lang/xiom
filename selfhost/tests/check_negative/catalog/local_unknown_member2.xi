// Phase 3 catalog gate: unknown member on a declared local module.
use dmod2;

fn main() -> Int {
  return dmod2.nope();
}
