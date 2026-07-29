module m21_vec_edge_012
type Item = { id: Int; val: Int; }

  pub fn run() -> Int {
    var v: Vec[Item] = [];
    v.push({ id: 1; val: 10; });
    v.push({ id: 2; val: 20; });
    if v.len() == 2 && v[0].id == 1 && v[1].val == 20 { return 0; }
    return 1;
  }
use m21_vec_edge_012.run;
fn main() -> Int { return run(); }
