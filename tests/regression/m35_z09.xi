// M35-Z09: type_alias_chain+generic+struct_derive+Option+Result+match+contract+compound_assign+module
type A = Int;
type B = A;
type C = B;
type Entry = { key: C; val: Int; } derive[Eq]
enum Lookup { Found(v: Int), Missing }
fn search[T](e: Entry, target: C) -> Option[Int]
  requires: e.key >= 0
{
  if e.key == target { return Some(e.val); }
  return None;
}
fn resolve(r: Option[Int]) -> Result[Int, Str] {
  match r { Some(v) => Ok(v), None => Err("nope") }
}
module index {
  pub fn find(e: Entry, t: C) -> Option[Int] { return search(e, t); }
  pub fn get(e: Entry, t: C) -> Result[Int, Str] { var o = find(e, t); return resolve(o); }
  pub fn default_val() -> C { return 0 as C; }
}
use index.find;
use index.get;
use index.default_val;
fn main() -> Int {
  var e1 = Entry{ key: 42; val: 99; };
  var o1 = find(e1, 42 as C);
  var o2 = find(e1, 7 as C);
  var r1 = get(e1, 42 as C);
  var r2 = get(e1, 7 as C);
  var chk = 0;
  match o1 { Some(v) => { if v == 99 { chk += 1; } } None => {} }
  match o2 { Some(_) => {} None => { chk += 1; } }
  match r1 { Ok(v) => { if v == 99 { chk += 1; } } Err(_) => {} }
  match r2 { Ok(_) => {} Err(_) => { chk += 1; } }
  if chk == 4 && default_val() == 0 { return 0; }
  return 1;
}
