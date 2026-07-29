module m21_module_008
pub type Value = { x: Int; }

  pub fn zero() -> Value {
    return { x: 0; };
  }

  pub fn inc(v: Value) -> Value {
    var copy: Value = v;
    copy.x = copy.x + 1;
    return copy;
  }

  pub fn run() -> Int {
    var v = zero();
    var v2 = inc(v);
    var v3 = inc(v2);
    if v3.x == 2 { return 0; }
    return 1;
  }
use m21_module_008.run;
fn main() -> Int { return run(); }
