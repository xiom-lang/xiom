// M35-V03: Vec[Int] first and last via index access
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  // Empty Vec: len 0
  if v.len() != 0 { return 1; }
  v.push(42);
  if v.len() != 1 { return 2; }
  if v[0] != 42 { return 3; }
  v.push(99);
  if v[0] != 42 { return 4; }
  if v[1] != 99 { return 5; }
  v.push(7);
  if v[0] != 42 { return 6; }
  if v[v.len() - 1] != 7 { return 7; }
  // Remove first and last
  v.remove(0);
  if v[0] != 99 { return 8; }
  v.remove(v.len() - 1);
  if v[0] != 99 { return 9; }
  if v.len() != 1 { return 10; }
  return 0;
}
