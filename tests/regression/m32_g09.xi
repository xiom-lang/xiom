// M32-G09: Nested generic types (Option wrapping Int and matching)
fn unwrap_int(o: Option[Int], def: Int) -> Int {
  match o { Some(v) => { return v; } None => { return def; } }
}
fn map_int(o: Option[Int]) -> Int {
  match o { Some(v) => { return v + 1; } None => { return -1; } }
}
fn main() -> Int {
  var a: Option[Int] = Some(42);
  var r = unwrap_int(a, 0);
  if r != 42 { return 1; }
  var m = map_int(a);
  if m != 43 { return 2; }
  var b: Option[Int] = None;
  var d = unwrap_int(b, 99);
  if d != 99 { return 3; }
  return 0;
}
