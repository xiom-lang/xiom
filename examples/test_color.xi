module a {
  pub type Color = {
    r: Int;
    g: Int;
    b: Int;
    a: Int;
  } derive[Eq, Clone]

  fn test_a() -> Int {
    var c = Color{ r: 255, g: 128, b: 64, a: 255 };
    if c.r == 255 { return 1; }
    return 0;
  }

  pub fn run_a() -> Int { return test_a(); }
}

use a.run_a;

fn main() -> Int { return run_a(); }
