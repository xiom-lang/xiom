module m21_vec_edge_028
enum Kind { A, B, C }

  pub fn run() -> Int {
    var v: Vec[Kind] = [Kind.A, Kind.B, Kind.C];
    if v.len() == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_028.run;
fn main() -> Int { return run(); }
