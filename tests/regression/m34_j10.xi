// M34-J10: Nested modules (3+ levels) — deep nesting
module l1 {
  pub fn v1() -> Int { return 1; }
  module l2 {
    pub fn v2() -> Int { return 2; }
    module l3 {
      pub fn v3() -> Int { return 3; }
      module l4 {
        pub fn v4() -> Int { return 4; }
        module l5 {
          pub fn v5() -> Int { return 5; }
        }
      }
    }
  }
}
use l1.v1;
use l1.l2.v2;
use l1.l2.l3.v3;
use l1.l2.l3.l4.v4;
use l1.l2.l3.l4.l5.v5;
fn chain(start: Int) -> Int {
  var s: Int = start;
  s = s + v1();
  s = s + v2();
  s = s + v3();
  s = s + v4();
  s = s + v5();
  return s;
}
fn main() -> Int {
  if chain(0) == 15 { return 0; }
  return 1;
}
