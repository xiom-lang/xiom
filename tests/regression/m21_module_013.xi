module m21_module_013
pub type Node = { val: Int; next: Option[Int]; }

  pub fn new_node(v: Int) -> Node { return { val: v; next: None; }; }

  pub fn run() -> Int {
    var n = new_node(5);
    if n.val == 5 { return 0; }
    return 1;
  }
use m21_module_013.run;
fn main() -> Int { return run(); }
