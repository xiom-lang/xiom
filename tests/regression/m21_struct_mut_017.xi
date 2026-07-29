module m21_struct_mut_017
type Entry = { key: Int; val: Int; }

  pub fn run() -> Int {
    var res: Result[Entry, Int] = Ok({ key: 1; val: 100; });
    match res {
      Ok(e) => {
        if e.key == 1 && e.val == 100 { return 0; }
      }
      Err(_) => { return 1; }
    }
    return 1;
  }
use m21_struct_mut_017.run;
fn main() -> Int { return run(); }
