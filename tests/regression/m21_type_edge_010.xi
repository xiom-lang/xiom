module m21_type_edge_010
type MyResult = Result[Int, Str]

  pub fn run() -> Int {
    var r: MyResult = Ok(42);
    match r {
      Ok(v) => if v == 42 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_type_edge_010.run;
fn main() -> Int { return run(); }
