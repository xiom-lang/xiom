module m21_match_edge_007
pub fn run() -> Int {
    var opt: Option[Int] = Some(15);
    match opt {
      Some(v) if v > 10 => return 0,
      Some(v) if v < 10 => return 1,
      Some(_) => return 2,
      None => return 3,
    }
  }
use m21_match_edge_007.run;
fn main() -> Int { return run(); }
