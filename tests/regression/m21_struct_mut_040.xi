module m21_struct_mut_040
type Entry = { key: Int; val: Str; }

  fn get_val(e: Entry) -> Int {
    return e.key;
  }

use m21_struct_mut_040.run;
fn main() -> Int { return run(); }
