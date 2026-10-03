// Phase 3 catalog gate, accept case: the `xiom.<mod>` namespace root forms a
// multi-segment module path (`xiom.math.shr`).
use xiom.math;

fn main() -> Int {
  var s = xiom.math.shr(16, 2);
  return s;
}
