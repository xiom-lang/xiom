module m21_vec_edge_026
pub fn run() -> Int {
    var v: Vec[Int] = [];
    var i = 0;
    while i < 100 {
      v.push(i);
      i = i + 1;
    }
    if v.len() == 100 && v[0] == 0 && v[99] == 99 { return 0; }
    return 1;
  }
use m21_vec_edge_026.run;
fn main() -> Int { return run(); }
