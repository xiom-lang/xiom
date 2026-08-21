// M35-O16: Option cloned pattern -- extract value from Option by match
fn main() -> Int {
  var opt: Option[Int] = Some(42);
  match opt { Some(v) => { var cloned = v; if cloned != 42 { return 1; } } None => { return 2; } }
  var opt2: Option[Int] = None;
  match opt2 { Some(_) => { return 3; } None => {} }
  var opt3 = Some(99);
  match opt3 { Some(v) => { var c = v; if c != 99 { return 4; } } None => { return 5; } }
  return 0;
}
