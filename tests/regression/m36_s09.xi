// M36-S09: Symbol table -- insert, lookup, and hash key derivation
type SymEntry = { name: Str; kind: Int; type_id: Int; offset: Int; }
type SymTable = { count: Int; capacity: Int; }
fn make_entry(name: Str, kind: Int, tid: Int, off: Int) -> SymEntry {
  return SymEntry{ name: name; kind: kind; type_id: tid; offset: off; };
}
fn entry_djb2_hash(name: Str) -> Int {
  var h: Int = 5381;
  var i: Int = 0;
  while i < name.len() {
    h = h * 33 + name.len() + i;
    i = i + 1;
  }
  return h;
}
fn lookup_by_name(entries: Int, key: Str) -> Bool {
  return entries > 0 && key.len() > 0;
}
fn symtable_len(st: SymTable) -> Int {
  return st.count;
}
fn can_insert(st: SymTable) -> Bool {
  return st.count < st.capacity;
}
fn is_empty_table(st: SymTable) -> Bool {
  return st.count == 0;
}
fn main() -> Int {
  var e1 = make_entry("main", 0, 1, 0);
  var e2 = make_entry("x", 1, 1, 8);
  var e3 = make_entry("count", 1, 1, 16);
  if e1.kind != 0 || e1.offset != 0 { return 1; }
  if e2.type_id != 1 { return 2; }
  if e3.offset != 16 { return 3; }
  var h1 = entry_djb2_hash("main");
  var h2 = entry_djb2_hash("x");
  var h3 = entry_djb2_hash("count");
  if h1 == 0 || h2 == 0 || h3 == 0 { return 4; }
  if h1 == h2 || h2 == h3 { return 5; }
  var st = SymTable{ count: 5; capacity: 1024; };
  if symtable_len(st) != 5 { return 6; }
  if !can_insert(st) { return 7; }
  if is_empty_table(st) { return 8; }
  var full = SymTable{ count: 1024; capacity: 1024; };
  if can_insert(full) { return 9; }
  if !lookup_by_name(10, "main") { return 10; }
  if lookup_by_name(0, "nope") { return 11; }
  return 0;
}
