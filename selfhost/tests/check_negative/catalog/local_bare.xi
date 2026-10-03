// Phase 3 catalog gate, accept case: bare exports of a local file resolve
// even when the file has no `module` header.
use dmod;

fn main() -> Int {
  return real();
}
