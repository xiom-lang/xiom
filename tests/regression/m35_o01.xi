// M35-O01: Option[Int] create -- Some and None construction with match
fn main() -> Int {
  var a: Option[Int] = Some(42);
  var b: Option[Int] = None;
  match a { Some(v) => { if v != 42 { return 1; } } None => { return 2; } }
  match b { Some(_) => { return 3; } None => {} }
  var c = Some(-7);
  match c { Some(v) => { if v != -7 { return 4; } } None => { return 5; } }
  var d: Option[Int] = Some(0);
  match d { Some(v) => { if v != 0 { return 6; } } None => { return 7; } }
  return 0;
}
