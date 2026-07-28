// M32-E02: Simple enum with wildcard catch-all
enum Status { Good, Warn, Bad }
fn main() -> Int {
  match Status.Good {
    Good => {}
    _ => { return 1; }
  }
  match Status.Warn {
    Good => { return 1; }
    Warn => {}
    _ => { return 1; }
  }
  match Status.Bad {
    Good => { return 1; }
    _ => {}
  }
  return 0;
}
