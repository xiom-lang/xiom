// M34-J11: Module re-export — pub use of items from another module
module internal {
  pub fn compute(x: Int) -> Int { return x * 10; }
  pub type Value = { raw: Int; }
  pub fn create(v: Int) -> Value { return Value{ raw: v; }; }
}
module api {
  use internal.compute;
  pub fn calc(x: Int) -> Int { return compute(x); }
  pub fn double_compute(x: Int) -> Int {
    var v = compute(x);
    return v + v;
  }
}
use api.calc;
use api.double_compute;
fn main() -> Int {
  var r1 = calc(5);
  var r2 = double_compute(3);
  if r1 == 50 && r2 == 60 { return 0; }
  return 1;
}
