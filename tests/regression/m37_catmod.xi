module m37_catmod
// Catalog module used by the BUG 22/23 cross-module regression tests.

pub fn mk_f(x: Float64) -> Float64 { return x + 1.0; }

pub fn mk_vecf() -> Vec[Float64] {
  var v = Vec[Float64].new();
  v.push(1.5); v.push(2.5);
  return v;
}

pub fn mk_matrix() -> Vec[Vec[Int]] {
  var m = Vec[Vec[Int]].new();
  var r1 = Vec[Int].new(); r1.push(1); r1.push(2);
  var r2 = Vec[Int].new(); r2.push(3); r2.push(4);
  m.push(r1); m.push(r2);
  return m;
}

pub fn mk_opt() -> Option[Int] { return Some(5); }
pub fn mk_optf() -> Option[Float64] { return Some(2.5); }

pub fn mk_optvec() -> Option[Vec[Int]] {
  var v = Vec[Int].new();
  v.push(7);
  v.push(8);
  return Some(v);
}

pub fn mk_bb(b1: Bool, b2: Bool) -> (Bool, Bool) { return (b1, b2); }

pub fn mk_t3(a: Int, b: Int, c: Int) -> (Int, Int, Int) { return (a, b, c); }

pub fn fill_ints(v: &Vec[Int]) {
  v.push(7);
  v.push(8);
}

pub fn fill_ints_mut(v: &mut Vec[Int]) {
  v.push(9);
}
