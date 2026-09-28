// m154 (Stage 6 W004): unreachable match arms must warn -- catch-all
// shadow, duplicate literal, duplicate enum variant. Compilation stays
// exit 0 and the program runs.
module p_w004_positive;

use xiom.io;

enum Color { Red, Green, Blue }

fn classify(n: Int) -> Str {
  match n {
    1 => { "one" },
    1 => { "uno" },
    _ => { "other" },
    2 => { "two" },
  }
}

fn shade(c: Color) -> Str {
  match c {
    Color.Red => { "r" },
    Color.Green => { "g" },
    Color.Green => { "g2" },
    Color.Blue => { "b" },
  }
}

fn binding_shadow(n: Int) -> Str {
  match n {
    x => { "any" },
    5 => { "five" },
  }
}

fn main() -> Int {
  io.println(classify(1));
  io.println(shade(Color.Red));
  io.println(binding_shadow(5));
  return 0;
}
