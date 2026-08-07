module m21_ffi_unsafe_007
pub fn run() -> Int {
    unsafe {
      var x: Int32 = 1000i32;
      var p: *Int32 = &x;
      if *p == 1000i32 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_007.run;
fn main() -> Int { return run(); }
