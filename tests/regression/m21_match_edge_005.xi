module m21_match_edge_005
enum State { One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten }

  pub fn run() -> Int {
    var s = State.Five;
    match s {
      State.One => return 1,
      State.Two => return 2,
      State.Three => return 3,
      State.Four => return 4,
      State.Five => return 0,
      State.Six => return 6,
      State.Seven => return 7,
      State.Eight => return 8,
      State.Nine => return 9,
      State.Ten => return 10,
    }
  }
use m21_match_edge_005.run;
fn main() -> Int { return run(); }
