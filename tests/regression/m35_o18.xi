// M35-O18: Option zip — combine two Options match-based (two Int values)
fn zip_both_some() -> Int {
  var a = Some(1);
  var b = Some(10);
  match a {
    Some(x) => { match b { Some(y) => { if x != 1 { return 1; } if y != 10 { return 2; } return 0; } None => { return 3; } } }
    None => { return 4; }
  }
}
fn zip_first_none() -> Int {
  var a: Option[Int] = None;
  var b = Some(20);
  match a { Some(_) => { return 1; } None => { match b { Some(_) => {} None => { return 2; } } } }
  return 0;
}
fn zip_second_none() -> Int {
  var a = Some(5);
  var b: Option[Int] = None;
  match a { Some(_) => { match b { Some(_) => { return 1; } None => {} } } None => { return 2; } }
  return 0;
}
fn zip_both_none() -> Int {
  var a: Option[Int] = None;
  var b: Option[Int] = None;
  match a { Some(_) => { return 1; } None => { match b { Some(_) => { return 2; } None => {} } } }
  return 0;
}
fn main() -> Int {
  if zip_both_some() != 0 { return 1; }
  if zip_first_none() != 0 { return 2; }
  if zip_second_none() != 0 { return 3; }
  if zip_both_none() != 0 { return 4; }
  return 0;
}
