module m21_struct_mut_037
type Box[T] = { val: T; }

fn main() -> Int {
    var b: Box[Int] = { val: 10; };
    b.val = 42;
    if b.val == 42 { return 0; }
    return 1;
  }
use m21_struct_mut_037.run;
fn main() -> Int { return run(); }
