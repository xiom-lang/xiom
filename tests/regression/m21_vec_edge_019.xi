module m21_vec_edge_019
pub fn run() -> Int {
    var outer: Vec[Vec[Int]] = [];
    var inner1: Vec[Int] = [];
    inner1.push(10);
    inner1.push(20);
    outer.push(inner1);
    outer[0].push(30);
    if outer[0].len() == 3 && outer[0][2] == 30 { return 0; }
    return 1;
  }
use m21_vec_edge_019.run;
fn main() -> Int { return run(); }
