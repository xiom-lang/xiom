// Phase 3 catalog-body gate: an imported local module with a body error is
// checked and its finding tagged `catalog body [<module>]: ` with the
// module's own line/col.
use fixture_broken;

fn main() -> Int {
  return fixture_broken.add(1, 2);
}
