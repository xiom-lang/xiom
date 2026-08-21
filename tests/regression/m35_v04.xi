// M35-V04: Vec[Int] is_empty -- verify empty/non-empty state
fn main() -> Int {
  var v = Vec[Int].new();
  if v.len() != 0 { return 1; }
  v.push(1);
  if v.len() == 0 { return 2; }
  v.push(2);
  if v.len() == 0 { return 3; }
  v.pop();
  if v.len() == 0 { return 4; }
  v.pop();
  if v.len() != 0 { return 5; }
  v.push(10);
  v.push(20);
  v.push(30);
  // Clear via pop loop
  while v.len() > 0 {
    v.pop();
  }
  if v.len() != 0 { return 6; }
  return 0;
}
