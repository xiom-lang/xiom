module m21_struct_mut_020
type Stats = { min: Int; max: Int; avg: Int; }

  pub fn run() -> Int {
    var s: Stats = { min: 0; max: 0; avg: 0; };
    s.min = 5;
    s.max = 95;
    s.avg = 50;
    if s.min == 5 && s.max == 95 && s.avg == 50 { return 0; }
    return 1;
  }
use m21_struct_mut_020.run;
fn main() -> Int { return run(); }
