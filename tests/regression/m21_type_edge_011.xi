module m21_type_edge_011
type MyOpt = Option[Int]

  pub fn run() -> Int {
    var o: MyOpt = Some(7);
    match o {
      Some(v) => if v == 7 { return 0; },
      None => return 1,
    }
  }
use m21_type_edge_011.run;
fn main() -> Int { return run(); }
