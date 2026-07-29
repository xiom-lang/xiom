module m21_match_edge_013
pub fn run() -> Int {
    var s = "beta";
    match s {
      "alpha" => return 1,
      "beta" => return 0,
      "gamma" => return 1,
      _ => return 1,
    }
  }
use m21_match_edge_013.run;
fn main() -> Int { return run(); }
