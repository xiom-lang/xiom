module m21_async_spawn_001
async fn delayed() -> Int {
    return 42;
  }

  pub fn run() -> Int {
    var x = 42;
    if x == 42 { return 0; }
    return 1;
  }
use m21_async_spawn_001.run;
fn main() -> Int { return run(); }
