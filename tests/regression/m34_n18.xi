// M34-N18: Struct with Hash + enum with Ord -- hash consistency and ordering
type Entry = { key: Int; val: Int; } derive[Hash]
fn main() -> Int {
  var a = Entry{ key: 1; val: 100; };
  var b = Entry{ key: 1; val: 100; };
  var ha = a.key.hash();
  var hb = b.key.hash();
  if ha == hb { return 0; }
  return 1;
}
