module m21_struct_mut_024
type Node = Box{ value: Int; next: Option[Int]; }

  pub fn run() -> Int {
    var n: Node = Box{ value: 1; next: None; };
    n.next = Some(99);
    match n.next {
      Some(v) => if v == 99 { return 0; },
      None => return 1,
    }
  }
use m21_struct_mut_024.run;
fn main() -> Int { return run(); }
