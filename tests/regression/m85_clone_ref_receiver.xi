// m85 (R40): derive[Clone] must mirror the callee ABI when the receiver is a
// pointer (`m: &M` -> `m.clone()`). Pre-fix the call passed the pointer where
// the by-value `%self` was expected; LLVM accepted the mismatch silently and
// the returned struct was garbage (clone on &T / &enum).
module m85_clone_ref_receiver

pub type Pair = {
  lo: Int;
  hi: Int;
} derive[Clone]

pub fn mk_pair() -> Pair {
  return Pair{ lo: 3, hi: 4 };
}

pub fn clone_sum(p: &Pair) -> Int {
  var c = p.clone();
  return c.lo + c.hi;
}

pub fn clone_sum_value(p: Pair) -> Int {
  var c = p.clone();
  return c.lo + c.hi;
}

pub enum Flag {
  Off
  On(code: Int)
} derive[Clone, Eq]

pub fn mk_flag() -> Flag {
  return On(7);
}

pub fn clone_matches(f: &Flag) -> Int {
  var g = f.clone();
  if g == On(7) { return 1; }
  return 0;
}

fn main() -> Int {
  var p = mk_pair();
  if clone_sum(&p) != 7 { return 1; }
  if clone_sum_value(p) != 7 { return 2; }
  var f = mk_flag();
  if clone_matches(&f) != 1 { return 3; }
  return 0;
}
