// M34-N2-12: 5-level nested modules — m1 → m2 → m3 → m4 → m5
module m1 {
  pub fn v1() -> Int { return 10; }
  module m2 {
    pub fn v2() -> Int { return 11; }
    module m3 {
      pub fn v3() -> Int { return 12; }
      module m4 {
        pub fn v4() -> Int { return 13; }
        module m5 {
          pub fn v5() -> Int { return 14; }
        }
      }
    }
  }
}
use m1.m2.m3.m4.m5.v5;
use m1.m2.m3.m4.v4;
use m1.m2.m3.v3;
use m1.m2.v2;
use m1.v1;
fn main() -> Int {
  var a = v1();
  var b = v2();
  var c = v3();
  var d = v4();
  var e = v5();
  if a == 10 && b == 11 && c == 12 && d == 13 && e == 14 { return 0; }
  return 1;
}
