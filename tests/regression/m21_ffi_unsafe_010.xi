module m21_ffi_unsafe_010
extern {
    fn strlen(s: *UInt8) -> Int;
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_ffi_unsafe_010.run;
fn main() -> Int { return run(); }
