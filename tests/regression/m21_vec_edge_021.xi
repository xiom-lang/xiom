module m21_vec_edge_021
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3];
    v.insert(1, 99);
    if v[0] == 1 && v[1] == 99 && v[2] == 2 && v[3] == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_021.run;
fn main() -> Int { return run(); }
