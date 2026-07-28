type Item = { id: Int; name: Option[Str]; }
fn describe(i: Item) -> Str {
  match i.name { Some(n) => n, None => "unnamed" }
}
fn main() -> Int {
  var a = Item{ id: 1; name: Some("hello"); };
  var b = Item{ id: 2; name: None; };
  if describe(a) != "hello" { return 1; }
  if describe(b) != "unnamed" { return 2; }
  return 0;
}
