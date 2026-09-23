// m119 (P1-4 AV/wrong answer): `is_sorted()`/`contains()` contract methods.
// The legacy lowering passed a pointer to the receiver VALUE to the
// xiom_is_sorted/xiom_contains runtime intrinsics, which read data[0] as the
// element COUNT -- for a `%struct.Vec` receiver that word is the DATA POINTER,
// so the scan walked arbitrary memory (deterministic access violation on the
// Windows CI runner, e2e_p1_contract_methods) and answered garbage everywhere
// else (a sorted [1..5] reported false). Both methods are now lowered INLINE
// over the real Vec header (len/data/elem-size) and unsupported element kinds
// fail loudly.
module m119_contract_method_values

type Bag = { items: Vec[Int]; }

fn main() -> Int {
  // 1. Vec[Int]: the exact e2e_p1_contract_methods shape.
  var arr = [1, 2, 3, 4, 5];
  if !arr.is_sorted() { return 1; }
  if !arr.contains(3) { return 2; }
  if arr.contains(9) { return 3; }

  var unsorted = [3, 1, 2];
  if unsorted.is_sorted() { return 4; }
  if !unsorted.contains(2) { return 5; }
  if unsorted.contains(9) { return 6; }

  // 2. Fixed-array binding (`let`) and array-literal receiver.
  let fixed = [1, 2, 3];
  if !fixed.is_sorted() { return 7; }
  if !fixed.contains(2) { return 8; }
  if ![1, 2, 3].is_sorted() { return 9; }
  if ![1, 2, 3].contains(3) { return 10; }

  // 3. Str elements (lexicographic order, content equality).
  var words = ["apple", "banana", "cherry"];
  if !words.is_sorted() { return 11; }
  if !words.contains("banana") { return 12; }
  var mixed = ["banana", "apple"];
  if mixed.is_sorted() { return 13; }
  if mixed.contains("zebra") { return 14; }

  // 4. Float64 elements.
  var floats = [1.5, 2.5, 3.0];
  if !floats.is_sorted() { return 15; }
  var desc = [3.0, 1.0];
  if desc.is_sorted() { return 16; }
  if !desc.contains(1.0) { return 17; }
  if desc.contains(2.5) { return 18; }

  // 5. Empty and single-element Vecs are trivially sorted.
  var empty: Vec[Int] = [];
  if !empty.is_sorted() { return 19; }
  if empty.contains(1) { return 20; }
  var one = [7];
  if !one.is_sorted() { return 21; }
  if !one.contains(7) { return 22; }

  // 6. Struct-field Vec receiver.
  var bag = Bag{ items: [1, 2, 3] };
  if !bag.items.is_sorted() { return 23; }
  if !bag.items.contains(2) { return 24; }

  return 0;
}
