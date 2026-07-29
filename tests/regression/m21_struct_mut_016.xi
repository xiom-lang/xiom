module m21_struct_mut_016
type Slot = Box{ value: Int; active: Bool; }

  pub fn run() -> Int {
    var s: Slot = Box{ value: 0; active: false; };
    s.value = 5;
    s.active = true;
    var new_s: Slot = Box{ value: s.value + 10; active: false; };
    if new_s.value == 15 && !new_s.active { return 0; }
    return 1;
  }
use m21_struct_mut_016.run;
fn main() -> Int { return run(); }
