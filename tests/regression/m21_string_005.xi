module m21_string_005
fn greet(name: Str) -> Str {
    return "Hello, ".concat(name);
  }

  pub fn run() -> Int {
    var msg = greet("World");
    if msg == "Hello, World" { return 0; }
    return 1;
  }
use m21_string_005.run;
fn main() -> Int { return run(); }
