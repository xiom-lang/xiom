module m21_vec_edge_016
pub fn run() -> Int {
    var v: Vec[Int] = [42];
    if v[0] == 42 { return 0; }
    return 1;
  }
use m21_vec_edge_016.run;
fn main() -> Int { return run(); }
