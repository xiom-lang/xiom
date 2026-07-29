module m21_vec_edge_006
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3];
    v.pop();
    v.pop();
    if v.len() == 1 { return 0; }
    return 1;
  }
use m21_vec_edge_006.run;
fn main() -> Int { return run(); }
