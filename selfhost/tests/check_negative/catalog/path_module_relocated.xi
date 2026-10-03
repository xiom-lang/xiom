// Phase 3 catalog gate, accept case: `xiom.path` lives at
// stdlib/xiom/os/path.xi; the relocation table resolves it by declared name.
use xiom.path;

fn main() -> Int {
  var sep = path.path_separator();
  return 0;
}
