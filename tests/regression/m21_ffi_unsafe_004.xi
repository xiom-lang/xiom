module m21_ffi_unsafe_004
type Data = { val: Int; }

  pub fn run() -> Int
  requires: true
{
    unsafe {
      var d: Data = { val: 99; };
      var p: *Data = &d;
      var v: Int = (*p).val;
      if v == 99 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_004.run;
fn main() -> Int { return run(); }
