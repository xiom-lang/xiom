module m21_vec_edge_030
fn make_vec() -> Vec[Int] {
    return [1, 2, 3, 4];
  }

  pub fn run() -> Int {
    var v = make_vec();
    if v.len() == 4 && v[0] == 1 && v[3] == 4 { return 0; }
    return 1;
  }
use m21_vec_edge_030.run;
fn main() -> Int { return run(); }
