// M36-S23: String interning pattern — hash-based lookup with deduplication
type InternEntry = { text: Str; id: Int; occupied: Bool; }
fn make_entry(text: Str, id: Int) -> InternEntry {
  return InternEntry{ text: text; id: id; occupied: true; };
}
fn empty_entry() -> InternEntry {
  return InternEntry{ text: ""; id: -1; occupied: false; };
}
fn is_occupied(e: InternEntry) -> Bool {
  return e.occupied;
}
fn intern_hash(s: Str, table_size: Int) -> Int {
  var h: Int = 5381;
  var i: Int = 0;
  while i < s.len() {
    h = (h * 33) + s.len() + i;
    i = i + 1;
  }
  if h < 0 { h = -h; }
  var idx = h % table_size;
  if idx < 0 { idx = idx + table_size; }
  return idx;
}
fn entry_matches(e: InternEntry, s: Str) -> Bool {
  return e.occupied && e.text == s;
}
fn next_id(last_id: Int) -> Int {
  return last_id + 1;
}
fn intern_slot(e: InternEntry, s: Str) -> Int {
  if !e.occupied { return 0; }
  if e.text == s { return 1; }
  return 2;
}
fn main() -> Int {
  var e1 = make_entry("main", 1);
  var e2 = make_entry("x", 2);
  var e3 = make_entry("count", 3);
  var e4 = empty_entry();
  if !is_occupied(e1) { return 1; }
  if !is_occupied(e3) { return 2; }
  if is_occupied(e4) { return 3; }
  var h1 = intern_hash("main", 256);
  var h2 = intern_hash("x", 256);
  var h3 = intern_hash("count", 256);
  if h1 == h2 { return 4; }
  if !entry_matches(e1, "main") { return 5; }
  if entry_matches(e1, "nope") { return 6; }
  if entry_matches(e4, "") { return 7; }
  if next_id(10) != 11 { return 8; }
  if next_id(0) != 1 { return 9; }
  var s1 = intern_slot(e1, "main");
  if s1 != 1 { return 10; }
  var s4 = intern_slot(e4, "new");
  if s4 != 0 { return 11; }
  var s_wrong = intern_slot(e1, "wrong");
  if s_wrong != 2 { return 12; }
  return 0;
}
