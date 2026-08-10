module m21_ffi_unsafe_001
pub fn run() -> Int
  requires: true
{
    unsafe {
      var x: Int = 42;
      var ptr: *Int = &x;
      if *ptr == 42 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_001.run;
fn main() -> Int { return run(); }
