// M35-L12: Option size pattern — verify Option layout via match
fn main() -> Int {
  var a: Option[Int] = Some(42);
  match a {
    Some(v) => if v != 42 { return 1; },
    None => { return 2; },
  }
  var b: Option[Int] = None;
  match b {
    Some(_) => { return 3; },
    None => {},
  }
  var c: Option[Int] = Some(100);
  match c {
    Some(v) => if v != 100 { return 4; },
    None => { return 5; },
  }
  var d: Option[Bool] = Some(true);
  match d {
    Some(v) => if v != true { return 6; },
    None => { return 7; },
  }
  return 0;
}
