module m21_vec_edge_010
pub fn run() -> Int {
    var v: Vec[Int8] = [];
    v.push(10i8);
    v.push(20i8);
    v.push(30i8);
    if v.len() == 3 && v[2] == 30i8 { return 0; }
    return 1;
  }
use m21_vec_edge_010.run;
fn main() -> Int { return run(); }
