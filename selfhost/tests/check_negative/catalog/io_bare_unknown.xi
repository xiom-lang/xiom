// Phase 3 catalog gate: a bare lowercase name stays undefined even when the
// file imports a module (Rust does not glob imports into bare scope).
use xiom.io;

fn main() -> Int {
  nope();
  return 0;
}
