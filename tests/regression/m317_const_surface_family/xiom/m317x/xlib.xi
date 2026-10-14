// m317 fixture module: the const module of the family (shapes a/c/d).
module xiom.m317x.xlib;

pub const VALUE: Int = 11;

// A second constant with a name the entry does NOT shadow: the bare-read
// shapes in xbare.xi use it so the lock stays free of the pre-existing
// bare-leaf keep-first ambiguity (a program-level const shadowing a
// HOMONYMOUS imported leaf still claims the flat leaf slot at codegen;
// record-only for m317, see COMPILER_BUGS).
pub const AVALUE: Int = 41;

pub fn xlib_get() -> Int {
  return VALUE;
}

fn dbl(x: Int) -> Int {
  return x * 2;
}

// Shape (d): a module-level `pub var` -- the parser silently dropped the
// `pub` pre-fix, so the export map never saw the var and consumers could
// not see it (T001).
pub var counter: Int = 7;

// Shape (d): a fn-valued module-level `pub var` called through the module.
pub var handler: fn(Int) -> Int = dbl;

pub fn call_handler(x: Int) -> Int {
  return handler(x);
}
