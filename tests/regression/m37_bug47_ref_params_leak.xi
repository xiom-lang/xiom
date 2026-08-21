// BUG 47 regression: `&T`-param names leaked across function compilations
// (ref_params/param_locals never cleared). A fn with `b: &Int` compiled
// before a fn whose VALUE param is also named `b` made `x.compare(&b)`
// dereference the value (inttoptr 7 -> load -> AV). Repro combines an impl
// method named `compare` used via a generic bound + a fn-typed param fn.
module m37_bug47_ref_params_leak
use xiom.sort.heap;

interface Ord9 {
  fn compare(other: &Self) -> Int;
}

impl Ord9 for Int {
  fn compare(self, other: &Int) -> Int {
    var s: Int = self;
    var o: Int = *other;
    if s < o { return -1; }
    if s > o { return 1; }
    return 0;
  }
}

fn max_of[T: Ord9](a: T, b: T) -> T {
  if a.compare(&b) >= 0 { return a; }
  return b;
}

// A fn whose PARAMS are named a/b with &Int types -- the leak source.
fn cmp_int(a: &Int, b: &Int) -> Int {
  if *a < *b { return -1; }
  if *a > *b { return 1; }
  return 0;
}

fn main() -> Int {
  var m = max_of[Int](3, 7);
  if m != 7 { return 1; }
  var hb = Vec[Int].new();
  hb.push(4); hb.push(1); hb.push(3); hb.push(2);
  heap_sort_by(&mut hb, cmp_int);
  if hb[0] != 1 || hb[3] != 4 { return 2; }
  var m2 = max_of[Int](9, 2);
  if m2 != 9 { return 3; }
  return 0;
}
