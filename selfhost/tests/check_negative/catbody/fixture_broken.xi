// Fixture for the catalog-body gate: a LOCAL module with a type error in its
// body (the Rust checker tags it `catalog body [fixture_broken]: ...`).
module fixture_broken

pub fn add(a: Int, b: Int) -> Int {
  return a + "s";
}
