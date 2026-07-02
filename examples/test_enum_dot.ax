module enums_mod2 {
  pub enum Dir { North, South }

  fn test() -> Int {
    if Dir.North != Dir.South { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return test(); }
}

use enums_mod2.run_test;

fn main() -> Int { return run_test(); }
