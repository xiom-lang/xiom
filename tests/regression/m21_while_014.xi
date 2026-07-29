module m21_while_014
fn done(i: Int) -> Bool { return i >= 10; }

  fn step(i: Int) -> Int { return i + 1; }

  pub fn run() -> Int {
    var i = 0;
    while !done(i) {
      i = step(i);
    }
    if i == 10 { return 0; }
    return 1;
  }
use m21_while_014.run;
fn main() -> Int { return run(); }
