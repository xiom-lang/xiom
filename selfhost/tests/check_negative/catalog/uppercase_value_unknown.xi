// Phase 3 imports gate: an unresolved uppercase name used as a value errors
// (Rust resolves module types through the import closure; unknown ones do
// not silently pass).
use xiom.io;

fn main() -> Int {
  var x = NoSuchThing;
  return 0;
}
