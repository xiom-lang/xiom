module m21_struct_mut_014
type Num = { val: Int; }

  fn Num.add(n: Int) {
    self.val = self.val + n;
  }

  fn Num.mul(n: Int) {
    self.val = self.val * n;
  }

fn main() -> Int {
    var x: Num = { val: 3; };
    x.add(7);
    x.mul(2);
    if x.val == 20 { return 0; }
    return 1;
  }
use m21_struct_mut_014.run;
fn main() -> Int { return run(); }
