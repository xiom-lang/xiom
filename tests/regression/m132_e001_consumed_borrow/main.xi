// m132 (E001 conservatism): temporary borrows consumed by a statement must
// not warn. Shape from stdlib smoke_collect_sparse: an immutable borrow in an
// `if` condition (`sparse_contains(&s, 5)`) followed by `&mut s` on a later
// statement warned 7 times although each borrow died with its call.
module m132_e001_consumed_borrow;

fn has(x: &Int) -> Bool { return *x > 0; }
fn add(x: &mut Int) { *x = *x + 1; }

fn main() -> Int {
  var s = 0;
  add(&mut s);
  if !has(&s) { return 1; }
  add(&mut s);
  if s != 2 { return 2; }
  if !has(&s) { return 3; }
  add(&mut s);
  if s != 3 { return 4; }
  return 0;
}
