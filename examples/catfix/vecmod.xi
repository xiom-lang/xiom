module vecmod
// BUG 8 fixture (docs/COMPILER_BUGS.md): an IMPORTED (catalog) module with
// `&Vec[Int]` and `&struct` params. Before the fix these catalog fns emitted
// empty/stubbed signatures (`define i64 @f()` with no params) while call sites
// passed `%struct.Vec*` — deterministic access violation.

pub type Wrap = { a: Int; b: Int; }

pub fn vec_sum(v: &Vec[Int]) -> Int {
  var s = 0;
  var i = 0;
  while i < v.len() {
    s = s + v[i];
    i = i + 1;
  }
  return s;
}

pub fn wrap_a(w: &Wrap) -> Int {
  return w.a;
}
