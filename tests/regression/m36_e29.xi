// M36-E29: Function with no statements -- just return
fn nop() -> Int { return 0; }
fn identity(x: Int) -> Int { return x; }
fn main() -> Int {
  if nop() != 0 { return 1; }
  if identity(42) != 42 { return 2; }
  if identity(-1) != -1 { return 3; }
  if identity(0) != 0 { return 4; }
  return 0;
}
