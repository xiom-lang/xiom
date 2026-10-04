// Phase 3 catalog gate: a resolved module member supplies its REAL return
// type (io.println -> Unit), so the return check fires instead of the call
// being treated as unknown.
use xiom.io;

fn main() -> Int {
  return io.println("x");
}
