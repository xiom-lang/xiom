// BUG 53 regression: &[N]T params — the caller's array-literal binding
// stored the Vec data POINTER as the array value (invalid IR: store
// [5 x i64] %ptr) and element access GEP'd the pointer SLOT as the
// array ([5 x i64]**, clang "invalid getelementptr indices"). Covers
// the generic &[N]T shape used by the stdlib array module.
module m37_bug53_array_ref_param
use xiom.array;

fn main() -> Int {
  var a: [5]Int = [10, 20, 30, 40, 50];
  if array.len(&a) != 5 { return 1; }
  if array.get(&a, 0).unwrap() != 10 { return 2; }
  if array.get(&a, 4).unwrap() != 50 { return 3; }
  return 0;
}
