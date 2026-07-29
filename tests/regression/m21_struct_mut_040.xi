module m21_struct_mut_040
type Entry = { key: Int; val: Str; }

  fn get_val(e: Entry) -> Int {
    return e.key;
  }

  pub fn run() -> Int {
    var e: Entry = { key: 42; val: "answer"; };
    var k = get_val(e);
    if k == 42 { return 0; }
    return 1;
  }
use m21_struct_mut_040.run;
fn main() -> Int { return run(); }
