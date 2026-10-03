// Phase 3 catalog gate, accept case: item imports (`use m.fn;`) make the
// imported function reachable by bare name.
use xiom.io.println;

fn main() -> Int {
  println("hi");
  return 0;
}
