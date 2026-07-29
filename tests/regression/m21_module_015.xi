module m21_module_015
pub type Data = { val: Int; }

  pub fn create(v: Int) -> Data { return { val: v; }; }

  pub fn get(d: Data) -> Int { return d.val; }

  pub fn run() -> Int {
    var d = create(77);
    var v = get(d);
    if v == 77 { return 0; }
    return 1;
  }
use m21_module_015.run;
fn main() -> Int { return run(); }
