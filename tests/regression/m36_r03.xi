// M36-R03: extra closing brace -- brace-depth stress with if-true nesting
fn main() -> Int {
  if true { if true { if true { return 0; } } };
  return 1;
}
