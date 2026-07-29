module m21_vec_edge_014
pub fn run() -> Int {
    var v: Vec[Result[Int, Int]] = [];
    v.push(Ok(1));
    v.push(Err(-1));
    v.push(Ok(3));
    if v.len() == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_014.run;
fn main() -> Int { return run(); }
