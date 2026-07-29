module m21_struct_mut_030
type Score = { val: Int; grade: Int; }

  pub fn run() -> Int {
    var s: Score = { val: 0; grade: 0; };
    var threshold = 50;
    if threshold > 40 {
      s.val = 75;
      s.grade = 2;
    } else {
      s.val = 25;
      s.grade = 1;
    }
    if s.val == 75 && s.grade == 2 { return 0; }
    return 1;
  }
use m21_struct_mut_030.run;
fn main() -> Int { return run(); }
