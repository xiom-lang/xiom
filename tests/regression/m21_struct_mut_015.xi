module m21_struct_mut_015
type Data = { val: Int; }

  fn maybe_get(data: Option[Data]) -> Data {
    match data {
      Some(d) => return d,
      None => return { val: -1; },
    }
  }

  pub fn run() -> Int {
    var d = maybe_get(Some({ val: 42; }));
    if d.val != 42 { return 1; }
    var d2 = maybe_get(None);
    if d2.val != -1 { return 1; }
    return 0;
  }

use m21_struct_mut_015.run;
fn main() -> Int { return run(); }
