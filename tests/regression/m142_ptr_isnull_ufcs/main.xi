// m142 (stdlib relay): method-position `ptr.is_null()`.
// `xiom.ptr.is_null[T](ptr: *const T)` is a free fn. Calling it as a method
// on a pointer FIELD (`h.p.is_null()`) used to emit no call at all, and on a
// `ptr` FIELD that shadows the `xiom.ptr` module alias (the stdlib's
// Ref/RefMut `release`) it emitted a garbage symbol (`@ptr.is_null`) which
// the auto-stub pass answered with a silent `ret 0` -- so every pointer read
// as non-null and `release` skipped the borrow decrement. Both method forms
// must agree with the direct call on null and non-null receivers.
module m142_ptr_isnull_ufcs;

use xiom.ptr;

pub type Holder = { p: *Int; }

// The stdlib shape: the `ptr` FIELD shadows the imported `xiom.ptr` module
// alias; the method form must bind the free fn with the field as arg 0.
pub type RefCell = { value: Int; borrows: Int; }
pub type Ref = { ptr: *mut RefCell; }

pub fn Ref.release(self)
  requires: true
{
  unsafe {
    if ptr.is_null() { return; }
    (*ptr).borrows = (*ptr).borrows - 1;
  };
}

// d = direct, m = method; packed d*10+m.
fn probe(h: &Holder) -> Int {
  let d = if ptr.is_null(h.p) { 1 } else { 0 };
  let m = if h.p.is_null() { 1 } else { 0 };
  return d * 10 + m;
}

fn main() -> Int {
  var x = 5;
  var live = Holder{ p: unsafe { &x as *Int } };
  var nullh = Holder{ p: unsafe { 0 as *Int } };

  // Null receiver: direct and method must BOTH be true (packed 11).
  if probe(&nullh) != 11 { return 1; }
  // Non-null control: both false (packed 0).
  if probe(&live) != 0 { return 2; }

  // Module-shadow shape: release must actually decrement the counter.
  var c = RefCell{ value: 7; borrows: 1 };
  var r = Ref{ ptr: ptr.from_mut(&mut c) };
  r.release();
  if c.borrows != 0 { return 3; }

  // Null handle: release must return early, leaving the counter untouched.
  var c2 = RefCell{ value: 9; borrows: 1 };
  var r2 = Ref{ ptr: unsafe { 0 as *mut RefCell } };
  r2.release();
  if c2.borrows != 1 { return 4; }

  return 0;
}
