module m21_async_spawn_002
spawn {
    var x = 1;
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_async_spawn_002.run;
fn main() -> Int { return run(); }
