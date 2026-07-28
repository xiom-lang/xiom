// M35-T18: Module importing with use
module m {
  pub fn answer() -> Int { return 42; }
}
use m.answer;
fn main() -> Int {
  if answer() != 42 { return 1; }
  return 0;
}

