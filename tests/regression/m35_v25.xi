// M35-V25: Vec map pattern -- transform each element
fn map_double(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    result.push(v[i] * 2);
    i = i + 1;
  }
  return result;
}

fn map_add_one(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    result.push(v[i] + 1);
    i = i + 1;
  }
  return result;
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.push(4);
  v.push(5);
  var doubled = map_double(&v);
  if doubled.len() != 5 { return 1; }
  if doubled[0] != 2 { return 2; }
  if doubled[1] != 4 { return 3; }
  if doubled[4] != 10 { return 4; }

  var inc = map_add_one(&v);
  if inc[0] != 2 { return 5; }
  if inc[4] != 6 { return 6; }

  // Original unchanged
  if v[0] != 1 { return 7; }
  if v[4] != 5 { return 8; }
  return 0;
}
