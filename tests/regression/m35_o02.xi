// M35-O02: Option[Float64] create -- Some/None construction and match extraction
fn main() -> Int {
  var a = Some(1.0);
  match a { Some(_) => {} None => { return 1; } }
  var b: Option[Float64] = None;
  match b { Some(_) => { return 2; } None => {} }
  var c = Some(0.0);
  match c { Some(_) => {} None => { return 3; } }
  var d: Option[Float64] = Some(2.0);
  match d { Some(_) => {} None => { return 4; } }
  return 0;
}
