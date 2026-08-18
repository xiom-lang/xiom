// BUG 49 regression: fn-typed params named like impl methods must call
// through the PASSED fn pointer, not the impl symbol. impl Eq[Int] /
// impl Ord[Int] register bare aliases "eq"/"compare"; a param named
// `compare` resolved to @Int.compare and the sort miscompiled (wrong order).
module m37_bug49_fn_param_impl_collision
use xiom.collections;

impl Eq[Int] {
  fn eq(a: Int, b: Int) -> Bool {
    return a == b;
  }
}

impl Ord[Int] {
  fn compare(a: Int, b: Int) -> Int {
    if a < b { return -1; }
    if a > b { return 1; }
    return 0;
  }
}

fn sort_by_cmp(v: &mut Vec[Int], compare: fn(&Int, &Int) -> Int) {
  var i = 0;
  while i < v.len() {
    var j = i + 1;
    while j < v.len() {
      if compare(&v[i], &v[j]) > 0 {
        var t = v[i];
        v[i] = v[j];
        v[j] = t;
      }
      j = j + 1;
    }
    i = i + 1;
  }
}

fn cmp_int(a: &Int, b: &Int) -> Int {
  if *a < *b { return -1; }
  if *a > *b { return 1; }
  return 0;
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(3);
  v.push(1);
  v.push(2);
  sort_by_cmp(&mut v, cmp_int);
  if v[0] != 1 { return 1; }
  if v[1] != 2 { return 2; }
  if v[2] != 3 { return 3; }
  return 0;
}
