module m21_struct_mut_033
type Value = { val: Int; flag: Bool; }

fn main() -> Int {
    var v: Value = { val: 10; flag: false; };
    if v.val > 5 {
      v.flag = true;
      v.val = v.val * 2;
    }
    if v.flag && v.val == 20 { return 0; }
    return 1;
  }
use m21_struct_mut_033.run;
fn main() -> Int { return run(); }
