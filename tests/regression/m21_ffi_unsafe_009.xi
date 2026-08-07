module m21_ffi_unsafe_009
type Node = { val: Int; next: *Node; }

  pub fn run() -> Int {
    unsafe {
      var n: Node = { val: 1; next: 0 as *Node; };
      if n.val == 1 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_009.run;
fn main() -> Int { return run(); }
