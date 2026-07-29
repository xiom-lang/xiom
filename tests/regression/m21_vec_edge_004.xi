module m21_vec_edge_004
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30];
    var val = v.pop();
    if val == 30 && v.len() == 2 { return 0; }
    return 1;
  }
use m21_vec_edge_004.run;
fn main() -> Int { return run(); }
