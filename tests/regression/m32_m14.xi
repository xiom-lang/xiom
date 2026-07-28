// M32-M14: Private fn vs pub fn — pub wraps private, only pub accessible
module container {
  fn secret(x: Int) -> Int { return x + 1; }
  pub fn public_api(x: Int) -> Int { return secret(x) * 2; }
  pub fn direct(x: Int) -> Int { return x; }
}
use container.public_api;
use container.direct;
fn main() -> Int {
  if public_api(10) == 22 && direct(7) == 7 { return 0; }
  return 1;
}
