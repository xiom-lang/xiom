module m21_vec_edge_018
pub fn run() -> Int {
    var outer: Vec[Vec[Int]] = [];
    outer.push([1, 2]);
    outer.push([3, 4, 5]);
    if outer.len() == 2 && outer[1].len() == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_018.run;
fn main() -> Int { return run(); }
