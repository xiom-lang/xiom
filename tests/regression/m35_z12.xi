// M35-Z12: unsafe+cast+const+generic+match+while+module+compound_assign+derive+Option+contract
const MAGIC: Int = 0xDEAD;
type Alloc = { size: Int; cap: Int; } derive[Eq]
fn check_alloc[T](a: Alloc) -> Int {
  if a.size > a.cap { return 0; }
  return 1;
}
fn dangerous_cast(v: Int) -> *Int {
  var p: *Int;
  unsafe { p = v as *Int; }
  return p;
}
module heap {
  pub fn verify(a: Alloc) -> Int { return check_alloc(a); }
  pub fn cast_ptr(v: Int) -> *Int { return dangerous_cast(v); }
  pub fn magic_val() -> Int { return MAGIC; }
}
use heap.verify;
use heap.cast_ptr;
use heap.magic_val;
fn main() -> Int {
  var al = Alloc{ size: 3; cap: 5; };
  var ok = verify(al);
  var p = cast_ptr(100);
  var chk = 0;
  var i = 0;
  while i < 4 { i += 1; }
  if i == 4 { chk += 1; }
  if ok == 1 { chk += 1; }
  if magic_val() == 0xDEAD { chk += 1; }
  if p != (0 as *Int) { chk += 1; }
  if chk == 4 { return 0; }
  return 1;
}
