// Phase 3 catalog-body gate: catalog-body findings flush BEFORE the user
// program's own diagnostics (Rust `flush_catalog_bodies`).
use fixture_broken;

fn main() -> Int {
  var x: Int = "s";
  return fixture_broken.add(1, 2);
}
