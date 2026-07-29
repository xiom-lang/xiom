module m21_type_edge_008
pub fn run() -> Int {
    var v: Vec[Option[Result[Int, Int]]] = [Some(Ok(1)), None, Some(Err(-1))];
    if v.len() == 3 { return 0; }
    return 1;
  }
use m21_type_edge_008.run;
fn main() -> Int { return run(); }
