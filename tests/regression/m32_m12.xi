// M32-M12: Dotted path for nested modules -- deep import via use
module alpha {
  pub fn get_a() -> Int { return 1; }
  module beta {
    pub fn get_b() -> Int { return 2; }
    module gamma {
      pub fn get_c() -> Int { return 3; }
    }
  }
}
use alpha.get_a;
use alpha.beta.get_b;
use alpha.beta.gamma.get_c;
fn main() -> Int {
  var a = get_a();
  var b = get_b();
  var c = get_c();
  if a == 1 && b == 2 && c == 3 { return 0; }
  return 1;
}
