// Phase 3 catalog gate, accept case: a declared local module resolves
// qualified calls through its exports.
use dmod2;

fn main() -> Int {
  return dmod2.real();
}
