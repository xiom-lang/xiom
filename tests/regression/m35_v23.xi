// M35-V23: Vec[Int] min/max — find min and max values
fn vec_min(v: &Vec[Int]) -> Option[Int] {
  if v.len() == 0 { return None; }
  var m = v[0];
  var i = 1;
  while i < v.len() {
    if v[i] < m { m = v[i]; }
    i = i + 1;
  }
  return Some(m);
}

fn vec_max(v: &Vec[Int]) -> Option[Int] {
  if v.len() == 0 { return None; }
  var m = v[0];
  var i = 1;
  while i < v.len() {
    if v[i] > m { m = v[i]; }
    i = i + 1;
  }
  return Some(m);
}

fn main() -> Int {
  var empty = Vec[Int].new();
  if vec_min(&empty) != None { return 1; }
  if vec_max(&empty) != None { return 2; }

  var v = Vec[Int].new();
  v.push(5);
  v.push(3);
  v.push(9);
  v.push(1);
  v.push(7);
  if vec_min(&v) != Some(1) { return 3; }
  if vec_max(&v) != Some(9) { return 4; }

  var v2 = Vec[Int].new();
  v2.push(42);
  if vec_min(&v2) != Some(42) { return 5; }
  if vec_max(&v2) != Some(42) { return 6; }

  return 0;
}
