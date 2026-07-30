module m21_struct_mut_029
type Accum = { sum: Int; count: Int; }

fn main() -> Int {
    var a: Accum = { sum: 0; count: 0; };
    var i = 0;
    while i < 10 {
      a.sum = a.sum + i;
      a.count = a.count + 1;
      i = i + 1;
    }
    if a.sum == 45 && a.count == 10 { return 0; }
    return 1;
  }
use m21_struct_mut_029.run;
fn main() -> Int { return run(); }
