// m123 (R70): `for x in <collection>`. The old lowering treated every
// iterable as Range{start,end}: for a %struct.Vec it read field 0 (the DATA
// POINTER) as the loop index and STORED data+1 back into it -- the loop then
// ran zero times (heap address > len) or corrupted the Vec's data pointer.
// The checker also bound the loop variable to Int unconditionally, so
// `for s in vec_of_str { str_len(s) }` failed with "expected Str, found Int".
// Collections now lower to a real element loop (Vec/Slice values, &Vec
// headers, fixed arrays, array literals) and the checker binds the element
// type; unsupported iterables are rejected loudly.
module m123_for_in_collections

use xiom.string;
use xiom.iter;

type Point = { x: Int; y: Int; }

fn sum_ref(v: &Vec[Int]) -> Int {
  var t = 0;
  for x in v { t = t + x; }
  return t;
}

fn longest(v: &Vec[Str]) -> Int {
  var best = 0;
  for s in v {
    var n = str_len(s);
    if n > best { best = n; }
  }
  return best;
}

fn main() -> Int {
  // 1. Vec local + &Vec param.
  var v = [10, 20, 30];
  if sum_ref(&v) != 60 { return 1; }
  var t = 0;
  for x in v { t = t + x; }
  if t != 60 { return 2; }

  // 2. The Vec's own data pointer must survive iteration.
  if v[2] != 30 { return 3; }

  // 3. Array literal iterable.
  var lt = 0;
  for x in [4, 5, 6] { lt = lt + x; }
  if lt != 15 { return 4; }

  // 4. Fixed-array binding.
  let fixed = [7, 8];
  var ft = 0;
  for x in fixed { ft = ft + x; }
  if ft != 15 { return 5; }

  // 5. Str elements (checker element type + codegen i8* load).
  var words = ["a", "bbb", "cc"];
  if longest(&words) != 3 { return 6; }

  // 6. break/continue + nested loops.
  var width = 0;
  for s in words {
    if str_len(s) == 1 { continue; }
    width = width + str_len(s);
    if width > 4 { break; }
  }
  if width != 5 { return 7; }
  var pairs = 0;
  for a in [1, 2] {
    for b in [10, 20] { pairs = pairs + a * b; }
  }
  if pairs != 90 { return 8; }

  // 7. Struct elements by value.
  var ps = [Point{ x: 1; y: 2 }, Point{ x: 3; y: 4 }];
  var pt = 0;
  for p in ps { pt = pt + p.x + p.y; }
  if pt != 10 { return 9; }

  // 8. A Range VALUE keeps {start,end} iteration, and a two-element fixed
  // array must NOT be mistaken for one.
  var r = range(2, 5);
  var rt = 0;
  for i in r { rt = rt + i; }
  if rt != 9 { return 10; }
  let two = [1, 2];
  var tt = 0;
  for x in two { tt = tt + x; }
  if tt != 3 { return 11; }

  // 9. Range SYNTAX desugars to range()/range_inclusive(); both must iterate
  // (range_inclusive previously compiled as an undefined i64 call).
  var incl = 0;
  for i in 0..=4 { incl = incl + i; }
  if incl != 10 { return 12; }
  var exc = 0;
  for i in 0..4 { exc = exc + i; }
  if exc != 6 { return 13; }

  return 0;
}
