module m21_vec_edge_005
pub fn run() -> Int {
    var v: Vec[Int] = [];
    var val = v.pop();
    match val {
      None => return 0,
      Some(_) => return 1,
    }
  }
use m21_vec_edge_005.run;
fn main() -> Int { return run(); }
