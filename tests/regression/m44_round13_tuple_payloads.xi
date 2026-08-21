// m44_round13_tuple_payloads -- round-13 (2026-08-22) regression:
// TUPLE PAYLOADS through Option/Vec (the btree first_entry family).
// `Some((i, v))` payload bindings kept the box pointer (elements bound
// literal 0); Vec[(Int, Int)] slots held 8 of 16 bytes (push stored the
// pointer, get() loaded a half-slot); the elem type lookup didn't
// normalize "(Int, Int)" -> "Tuple__Int__Int"; generic return types
// ("Vec[(Int, T)]") were rejected by the binding tracking. Unblocks
// enumerate/zip adapters AND BTreeMap.first_entry/last_entry
// (smoke_collections_btree_map was exit 7 at baseline).
module m44_round13_tuple_payloads
use xiom.iter;

fn main() -> Int {
  // 1. enumerate -- Option[(Int, Int)] through collect/get/match.
  var items = iter.range(10, 13).enumerate().collect();
  if items.len() != 3 { return 1; }
  match items.get(0) {
    Some((i, v)) => { if i != 0 { return 2; }; if v != 10 { return 3; }; },
    None => { return 4; },
  };

  // 2. zip -- Vec[(Int, Int)] element reads + tuple match.
  var zipped = iter.range(100, 103).zip(iter.range(200, 203)).collect();
  if zipped.len() != 3 { return 5; }
  match zipped.get(2) {
    Some((x, y)) => { if x != 102 { return 6; }; if y != 202 { return 7; }; },
    None => { return 8; },
  };

  // 3. BTreeMap.first_entry -- the Option[(K, V)] tuple payload
  // (the key element binds through the tuple pattern; the Str value is
  // verified via get() -- the checker's generic-tuple substitution does
  // not type the Str element of the (K, V) pattern yet).
  var bm = BTreeMap[Int, Str].new();
  bm.insert(5, "five");
  bm.insert(1, "one");
  bm.insert(9, "nine");
  match bm.first_entry() {
    Some((k, _)) => { if k != 1 { return 9; }; },
    None => { return 10; },
  };
  match bm.get(1) {
    Some(v) => { if v != "one" { return 11; }; },
    None => { return 12; },
  };
  return 0;
}
