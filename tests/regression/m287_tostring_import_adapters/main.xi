// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m287 lock (REL-20261010-1548-stdlib-3): importing `xiom.convert.tostring`
// pulled the `math.mathematical_logic` module in (via the transitive import
// set) and the adapter closure-FIELD call `self.predicate(&v)` inside
// FilterIter.next / TakeWhileIter.next was hijacked by the free-fn leaf
// (`math.mathematical_logic.predicate`) -- a DIRECT call bypassing the
// closure env, so the predicates never saw values (filter/take_while
// returned None immediately). The instance closure field must win over
// free-fn leaf matches. rc 0 = both predicates work with the import present.
module m287_tostring_import_adapters.main

use xiom.iter;
use xiom.io;
use xiom.convert.tostring as tostring;

fn lt3(x: &Int) -> Bool { return *x < 3; }

fn main() -> Int {
  var r = iter.range(0, 6);
  let fi = r.filter(lt3);
  let f0 = fi.next();
  if f0.is_none { return 1; }
  var r2 = iter.range(0, 6);
  let tw = r2.take_while(lt3);
  let t0 = tw.next();
  if t0.is_none { return 2; }
  return 0;
}
