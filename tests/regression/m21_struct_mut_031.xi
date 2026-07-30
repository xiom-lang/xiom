module m21_struct_mut_031
type Counter = { val: Int; }

  fn Counter.inc_and_get() -> Int {
    self.val = self.val + 1;
    return self.val;
  }

fn main() -> Int {
    var c: Counter = { val: 0; };
    var a = c.inc_and_get();
    var b = c.inc_and_get();
    if a == 1 && b == 2 && c.val == 2 { return 0; }
    return 1;
  }
use m21_struct_mut_031.run;
fn main() -> Int { return run(); }
