module m21_deep_expr_013
pub fn run() -> Int {
    var v: Vec[Int] = [];
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    v.push(9);
    v.push(10);
    if v.len() == 10 && v[0] + v[1] + v[2] + v[3] + v[4] + v[5] + v[6] + v[7] + v[8] + v[9] == 55 { return 0; }
    return 1;
  }
use m21_deep_expr_013.run;
fn main() -> Int { return run(); }
