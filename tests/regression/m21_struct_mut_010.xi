module m21_struct_mut_010
type Holder = { data: Vec[Int]; sum: Int; }

  pub fn run() -> Int {
    var h: Holder = { data: [10, 20, 30]; sum: 0; };
    h.sum = h.data.len();
    if h.sum == 3 { return 0; }
    return 1;
  }
use m21_struct_mut_010.run;
fn main() -> Int { return run(); }
