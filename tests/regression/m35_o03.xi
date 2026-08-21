// M35-O03: Option[Str] create -- Some/None with string values
fn main() -> Int {
  var a = Some("hello");
  var b: Option[Str] = None;
  match a { Some(v) => { if v != "hello" { return 1; } } None => { return 2; } }
  match b { Some(_) => { return 3; } None => {} }
  var c = Some("");
  match c { Some(v) => { if v != "" { return 4; } } None => { return 5; } }
  var d: Option[Str] = Some("XIOM");
  match d { Some(v) => { if v != "XIOM" { return 6; } } None => { return 7; } }
  return 0;
}
