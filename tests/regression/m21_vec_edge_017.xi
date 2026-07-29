module m21_vec_edge_017
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3, 4, 5];
    var last = v.len() - 1;
    if v[last] == 5 { return 0; }
    return 1;
  }
use m21_vec_edge_017.run;
fn main() -> Int { return run(); }
