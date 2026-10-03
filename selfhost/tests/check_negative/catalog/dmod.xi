// Fixture: sibling module WITHOUT a `module` declaration. Rust still lets
// bare `real()` resolve through the file, but `dmod.real` does not
// (<catalog/local_known_member.xi> pins the "cannot call").
pub fn real() -> Int {
  return 1;
}
