// Phase 3 catalog gate: a sibling file without a `module` header still
// resolves qualified calls under the FILE-STEM identity (ee7ab150 batch;
// both stem and declared names resolve, so `dmod.real()` is accepted).
use dmod;

fn main() -> Int {
  return dmod.real();
}
