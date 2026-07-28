// M36-E19: Deeply nested parentheses — 20 levels
fn main() -> Int {
  var result = ((((((((((((((((((((1 + 1))))))))))))))))))));
  if result != 2 { return 1; }
  return 0;
}