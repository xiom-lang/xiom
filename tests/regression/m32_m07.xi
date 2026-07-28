// M32-M07: pub const — public constant declared in module
module config {
  pub const MAX_SIZE: Int = 256;
}
fn main() -> Int {
  if config.MAX_SIZE == 256 { return 0; }
  return 1;
}
