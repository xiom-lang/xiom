module m21_vec_edge_027
type Handler = { id: Int; }

  fn make_handler(i: Int) -> Handler {
    return { id: i; };
  }

  pub fn run() -> Int {
    var v: Vec[Handler] = [];
    v.push(make_handler(1));
    v.push(make_handler(2));
    if v[0].id == 1 && v[1].id == 2 { return 0; }
    return 1;
  }
use m21_vec_edge_027.run;
fn main() -> Int { return run(); }
