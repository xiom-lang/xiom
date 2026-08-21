// M35-V15: Vec[Option[Int]] -- Vec of optional integers, len verification
fn main() -> Int {
  var v = Vec[Option[Int]].new();
  v.push(Some(10));
  v.push(None);
  v.push(Some(30));
  if v.len() != 3 { return 1; }
  v.push(Some(40));
  if v.len() != 4 { return 2; }
  v.pop();
  if v.len() != 3 { return 3; }
  v.pop();
  if v.len() != 2 { return 4; }
  v.pop();
  if v.len() != 1 { return 5; }
  v.pop();
  if v.len() != 0 { return 6; }
  return 0;
}
