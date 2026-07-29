module m21_module_010
pub var MAX_SIZE: Int = 100;
  pub var DEFAULT_NAME: Str = "unnamed";

  pub fn run() -> Int {
    if MAX_SIZE == 100 { return 0; }
    return 1;
  }
use m21_module_010.run;
fn main() -> Int { return run(); }
