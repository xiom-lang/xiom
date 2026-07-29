module m21_vec_edge_029
fn sum_vec(v: Vec[Int]) -> Int {
    var total = 0;
    var i = 0;
    while i < v.len() {
      total = total + v[i];
      i = i + 1;
    }
    return total;
  }

  pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30];
    var s = sum_vec(v);
    if s == 60 { return 0; }
    return 1;
  }
use m21_vec_edge_029.run;
fn main() -> Int { return run(); }
