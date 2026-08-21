// M35-V06: Vec[Int] insert -- insert elements at position
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(30);
  v.insert(1, 20);
  if v.len() != 3 { return 1; }
  if v[0] != 10 { return 2; }
  if v[1] != 20 { return 3; }
  if v[2] != 30 { return 4; }
  v.remove(1);
  v.insert(0, 5);
  if v[0] != 5 { return 5; }
  if v[1] != 10 { return 6; }
  if v.len() != 3 { return 7; }
  v.remove(2);
  v.insert(2, 40);
  if v[v.len() - 1] != 40 { return 8; }
  if v.len() != 3 { return 9; }
  return 0;
}
