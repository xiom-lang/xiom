// BUG 55 regression: unsafe-block context capture of POINTER-typed
// variables -- passing `p: *Int` across fn boundaries deref'd it (the
// callee received x's VALUE as the pointer); and Option/Result ctors
// inside confined-unsafe block fns built the GENERIC %struct.Option
// while the fn returns the concrete Option__Rc (payload-slot mismatch
// -> the Option/Result payload corruption family: Vec.get, Weak.upgrade,
// json_set, contains-style reads).
module m37_bug55_unsafe_ptr_capture
use xiom.rc;

fn main() -> Int {
  // (a) raw-pointer round-trip through fn boundaries
  var x: Int = 0;
  var p: *Int = &x;
  unsafe {
    *p = 42;
  }
  unsafe {
    if *p != 42 { return 1; }
  }
  // (b) Option[Struct] payload through an unsafe block (Weak.upgrade shape)
  var r = rc.Rc.new(100);
  var w = r.downgrade();
  match w.upgrade() {
    Some(up) => {
      if up.get() != 100 { return 2; }
    }
    None => { return 3; }
  }
  return 0;
}
