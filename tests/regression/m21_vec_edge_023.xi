module m21_vec_edge_023
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3];
    v.clear();
    if v.len() == 0 { return 0; }
    return 1;
  }
use m21_vec_edge_023.run;
fn main() -> Int { return run(); }
