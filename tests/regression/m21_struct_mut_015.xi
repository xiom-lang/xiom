module m21_struct_mut_015
type Data = { val: Int; }

  fn maybe_get(data: Option[Data]) -> Data {
    match data {
      Some(d) => return d,
      None => return { val: -1; },
    }
  }

use m21_struct_mut_015.run;
fn main() -> Int { return run(); }
