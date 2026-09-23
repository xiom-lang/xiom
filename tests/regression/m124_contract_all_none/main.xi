// m124 (P1-4 contract methods): `all(predicate)`/`none(predicate)` on a
// collection receiver. The legacy lowering called the xiom_all/xiom_none
// runtime stubs with len=0, so the method form silently returned true for
// every collection; and when the stdlib `core.all`/`core.none` helpers were
// registered (any program importing the core module), the call resolved to
// the generic helper, which is never monomorphised for the method shape and
// landed on the emitter's returning-zero auto-stub (method form was silently
// false). Both now lower INLINE over the Vec header: the predicate is a
// closure value (env[0] = code pointer), elements are passed as the i64 the
// closure ABI uses, and the scan is fail-fast. Plain function NAMES are
// rejected loudly (they are not closure values in this position).
module m124_contract_all_none

use xiom.string;

type Bag = { items: Vec[Int]; }

fn main() -> Int {
  var v = [2, 4, 6];

  // 1. Closure predicates, by value.
  if !v.all(fn(x: Int) -> Bool { return x > 0; }) { return 1; }
  if v.all(fn(x: Int) -> Bool { return x > 4; }) { return 2; }
  if !v.none(fn(x: Int) -> Bool { return x > 100; }) { return 3; }
  if v.none(fn(x: Int) -> Bool { return x > 4; }) { return 4; }

  // 2. Pipe closures.
  if !v.all(|x| x % 2 == 0) { return 5; }
  if v.all(|x| x % 2 == 1) { return 6; }
  if !v.none(|x| x < 0) { return 7; }

  // 3. Fail-fast: the last element violates `all`.
  if v.all(fn(x: Int) -> Bool { return x != 6; }) { return 8; }
  if v.none(fn(x: Int) -> Bool { return x == 6; }) { return 9; }

  // 4. Empty collections are vacuously true for both.
  var empty: Vec[Int] = [];
  if !empty.all(fn(x: Int) -> Bool { return false; }) { return 10; }
  if !empty.none(fn(x: Int) -> Bool { return true; }) { return 11; }

  // 5. Str elements (pointer bits carried through the i64 closure ABI).
  var words = ["aa", "bbb"];
  if !words.none(fn(s: Str) -> Bool { return s == "zz"; }) { return 12; }
  if !words.all(fn(s: Str) -> Bool { return str_len(s) > 0; }) { return 13; }
  if !words.all(fn(s: Str) -> Bool { return str_len(s) < 4; }) { return 14; }

  // 6. Struct-field receiver.
  var bag = Bag{ items: [1, 3, 5] };
  if !bag.items.all(|x| x > 0) { return 15; }
  if bag.items.none(|x| x > 0) { return 16; }

  // 7. contains/is_sorted neighbours keep working on the same receiver.
  if !v.contains(4) { return 17; }
  if !v.is_sorted() { return 18; }

  return 0;
}
