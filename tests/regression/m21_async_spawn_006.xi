module m21_async_spawn_006
spawn {
    var x = 1;
    spawn {
      var y = 2;
    }
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_async_spawn_006.run;
fn main() -> Int { return run(); }
