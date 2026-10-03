// Phase 3 catalog gate: a file without a `module` header does not resolve as
// a qualified module (R49-1: declared identity), even though bare exports do.
use dmod;

fn main() -> Int {
  return dmod.real();
}
