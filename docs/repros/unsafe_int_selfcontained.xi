// Self-contained repro of the unsafe-return pattern (BUG 27 #14):
// extern "C" fn returning Int32, cast to Int, returned from an unsafe block
// through the trampoline. fs_move's failure: rc always read non-zero even
// when the underlying call succeeded.
module unsafe_int_selfcontained

extern "C" {
  fn abs(x: Int32) -> Int32;
  fn strlen(s: *UInt8) -> Int32;
}

pub fn abs_rc(x: Int32) -> Int {
  unsafe {
    var r = abs(x) as Int;
    return r;
  }
}

pub fn strlen_rc(s: *UInt8) -> Int {
  unsafe {
    var r = strlen(s) as Int;
    return r;
  }
}

