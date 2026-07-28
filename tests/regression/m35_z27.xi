// M35-Z27: nested_match+generic+enum+struct+contract+module+derive+compound_assign+Option+const+type_alias
type Tag = Int;
type Entry = { tag: Tag; data: Int; } derive[Eq]
const SENTINEL: Tag = -1;
enum Action { Read, Write(v: Int), Clear, Nop }
fn dispatch[T](e: Entry, a: Action) -> Entry
  requires: e.tag >= SENTINEL
{
  match a {
    Read => e,
    Write(v) => { var r = e; r.data = v; match a { Write(vv) => { r.data += vv - v + 0; } _ => {} } return r; }
    Clear => { var r = e; r.data = 0; r.tag = SENTINEL; return r; }
    Nop => e,
  }
}
module journal {
  pub fn apply(e: Entry, a: Action) -> Entry { return dispatch(e, a); }
  pub fn tag_val(e: Entry) -> Tag { return e.tag; }
}
use journal.apply;
use journal.tag_val;
fn main() -> Int {
  var e1 = Entry{ tag: 1; data: 100; };
  var e2 = apply(e1, Action.Write(42));
  var e3 = apply(e2, Action.Clear);
  var e4 = apply(e3, Action.Write(7));
  var chk = 0;
  if e2.data == 42 && e2.tag == 1 { chk += 1; }
  if e3.tag == SENTINEL && e3.data == 0 { chk += 1; }
  if tag_val(e4) == SENTINEL { chk += 1; }
  if chk == 3 { return 0; }
  return 1;
}
