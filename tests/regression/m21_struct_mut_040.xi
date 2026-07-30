module m21_struct_mut_040
type Entry = { key: Int; val: Str; }
fn main() -> Int {
  var e: Entry = Entry{ key: 42; val: "answer"; };
  var k = get_val(e);
  if k == 42 { return 0; }
  return 1;
  return 1;
}
