module m21_struct_mut_021
type Flag = { active: Bool; ready: Bool; }

  pub fn run() -> Int {
    var f: Flag = { active: false; ready: false; };
    f.active = true;
    f.ready = true;
    if f.active && f.ready { return 0; }
    return 1;
  }
use m21_struct_mut_021.run;
fn main() -> Int { return run(); }
