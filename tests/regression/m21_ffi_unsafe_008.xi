module m21_ffi_unsafe_008
pub fn run() -> Int {
    unsafe {
      var arr: Vec[Int] = [10, 20, 30];
      var p: *Int = &arr[0];
      if *p == 10 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_008.run;
fn main() -> Int { return run(); }
