// m43_round13_closure_adapters -- round-13 (2026-08-22) regression:
// the closure-based iterator adapter family (part 1: adapters + enum
// scrutinees). Covers the compiler roots that blocked the adapters:
// (1) closure env STRUCT NAME collisions -- identical capture shapes in
//     different mono fns redefined %struct.__closure_env_N (clang error);
// (2) captured-state MUTATION did not persist -- the next-closure's
//     captured Range advanced only a local copy (count/fold hung);
// (3) FN-TYPED FIELD calls (`self.next_fn()`, `self.f(v)`) compiled to
//     zero-param stubs -- the field holds a closure ENV, not a code ptr
//     (0xC000001D in MapIter.next/FilterIter.next);
// (5) cmp.min_by's Ordering-return scrutinee (match discriminant checks
//     missing for call scrutinees -- always returned the last arm).
// NOTE: the combined enumerate/zip/btree fixture is split into
// m44_round13_tuple_payloads.xi -- the single combined module flips the
// documented clang -O2 / MSVC-CRT startup crash (m34_y15/y20 family).
module m43_round13_closure_adapters
use xiom.iter;
use xiom.cmp;

fn main() -> Int {
  // 1. filter + map chain (fn-typed field calls + captured state).
  var mapped = iter.range(1, 10)
    .filter(fn(x: &Int) -> Bool { return *x % 2 == 0; })
    .map(fn(x: Int) -> Int { return x * 10; })
    .collect();
  if mapped.len() != 4 { return 1; }
  match mapped.get(0) { Some(v) => { if v != 20 { return 2; } }, None => { return 3; }, };

  // 2. take/skip countdown persists across closure invocations.
  var taken = iter.range(1, 10).take(3).collect();
  if taken.len() != 3 { return 4; }
  match taken.get(2) { Some(v) => { if v != 3 { return 5; } }, None => { return 6; }, };
  var skipped = iter.range(1, 6).skip(3).collect();
  if skipped.len() != 2 { return 7; }
  match skipped.get(0) { Some(v) => { if v != 4 { return 8; } }, None => { return 9; }, };

  // 3. chain (single-consumer closures -- no double-capture of r2).
  var chained = iter.range(1, 4).chain(iter.range(10, 13)).collect();
  if chained.len() != 6 { return 10; }
  match chained.get(3) { Some(v) => { if v != 10 { return 11; } }, None => { return 12; }, };

  // 4. cmp.min_by with a closure comparator (enum-return scrutinee).
  var r = cmp.min_by(10, 20, fn(a: &Int, b: &Int) -> cmp.Ordering {
    if *a < *b { return cmp.Less; };
    if *a > *b { return cmp.Greater; };
    return cmp.Equal;
  });
  if r != 10 { return 13; }
  return 0;
}
