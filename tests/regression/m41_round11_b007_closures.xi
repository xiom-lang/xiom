// m41_round11_b007_closures -- round-11 (2026-08-20) regression:
// B-007 closures -- fn-typed PARAMS hold a closure ENV pointer (field 0 =
// the fn ptr). Calling `f(x)` inside a generic body must go through the
// M20-A1 closure path: (1) the param must be registered as a closure local
// (otherwise the ENV pointer was inttoptr'd as a CODE pointer -- 0xC0000005
// in Option.map), and (2) the closure's REAL return type drives the
// fn-pointer signature -- struct returns (Option[Int]) are BY VALUE, never
// pointer derefs (0xC0000005 in Option.and_then).
module m41_round11_b007_closures
use xiom.core;

fn main() -> Int {
  // 1. Option.map with a fn-literal arg.
  let s = Some(10);
  let mapped = s.map(fn(x: Int) -> Int { return x * 3; });
  match mapped {
    Some(v) => { if v != 30 { return 1; } }
    None => { return 2; }
  }
  // 2. None map stays None.
  let n: Option[Int] = None;
  let mapped_none = n.map(fn(x: Int) -> Int { return x * 3; });
  if mapped_none.is_some { return 3; }
  // 3. unwrap_or_else with a ZERO-ARG closure.
  if s.unwrap_or_else(fn() -> Int { return 0; }) != 10 { return 4; }
  // 4. and_then with a closure returning Option[Int] (struct return).
  if s.and_then(fn(x: Int) -> Option[Int] { return Some(x * 2); }).unwrap_or(0) != 20 { return 5; }
  let n2: Option[Int] = None;
  if n2.and_then(fn(x: Int) -> Option[Int] { return Some(x * 2); }).is_some { return 6; }
  // 5. filter with a predicate closure (ref param per the stdlib signature).
  let f = Some(5);
  let filt = f.filter(fn(x: &Int) -> Bool { return *x > 3; });
  if filt.is_none { return 7; }
  let filt2 = f.filter(fn(x: &Int) -> Bool { return *x > 10; });
  if filt2.is_some { return 8; }
  return 0;
}
