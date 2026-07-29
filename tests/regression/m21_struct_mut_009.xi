module m21_struct_mut_009
type Container = { items: Vec[Int]; label: Int; }

  pub fn run() -> Int {
    var c: Container = { items: []; label: 0; };
    c.items.push(1);
    c.items.push(2);
    c.items.push(3);
    c.label = 7;
    if c.items.len() == 3 && c.label == 7 { return 0; }
    return 1;
  }
use m21_struct_mut_009.run;
fn main() -> Int { return run(); }
