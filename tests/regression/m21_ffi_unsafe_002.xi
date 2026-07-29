module m21_ffi_unsafe_002
type Point = { x: Int; y: Int; }

  pub fn run() -> Int {
    var p: Point = { x: 10; y: 20; };
    unsafe {
      var ptr: *Point = &p;
      if (*ptr).x == 10 { return 0; }
    }
    return 1;
  }
use m21_ffi_unsafe_002.run;
fn main() -> Int { return run(); }
