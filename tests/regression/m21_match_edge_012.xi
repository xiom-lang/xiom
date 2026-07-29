module m21_match_edge_012
type Point = { x: Int; y: Int; }

  pub fn run() -> Int {
    var p: Point = { x: 10; y: 20; };
    match p.x {
      10 => return 0,
      _ => return 1,
    }
  }
use m21_match_edge_012.run;
fn main() -> Int { return run(); }
