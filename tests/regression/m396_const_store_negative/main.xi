// m396 lock: a module-level `const` cannot be reassigned.
const LIMIT: Int = 3;

fn main() -> Int {
  LIMIT = 4;
  return 0;
}
