module m21_struct_mut_008
type Named = Person{ name: Str; id: Int; }

  pub fn run() -> Int {
    var n: Named = Person{ name: "alpha"; id: 0; };
    n.name = "beta";
    n.id = 99;
    if n.id == 99 { return 0; }
    return 1;
  }
use m21_struct_mut_008.run;
fn main() -> Int { return run(); }
