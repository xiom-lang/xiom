// M35-V26: Vec fold/sum pattern -- accumulate over Vec
fn vec_sum(v: &Vec[Int]) -> Int {
  var acc = 0;
  var i = 0;
  while i < v.len() {
    acc = acc + v[i];
    i = i + 1;
  }
  return acc;
}

fn vec_product(v: &Vec[Int]) -> Int {
  if v.len() == 0 { return 1; }
  var acc = 1;
  var i = 0;
  while i < v.len() {
    acc = acc * v[i];
    i = i + 1;
  }
  return acc;
}

fn vec_count_match(v: &Vec[Int], target: Int) -> Int {
  var acc = 0;
  var i = 0;
  while i < v.len() {
    if v[i] == target { acc = acc + 1; }
    i = i + 1;
  }
  return acc;
}

fn main() -> Int {
  var empty = Vec[Int].new();
  if vec_sum(&empty) != 0 { return 1; }
  if vec_product(&empty) != 1 { return 2; }

  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.push(4);
  v.push(5);
  if vec_sum(&v) != 15 { return 3; }
  if vec_product(&v) != 120 { return 4; }

  var v2 = Vec[Int].new();
  v2.push(7);
  v2.push(7);
  v2.push(3);
  v2.push(7);
  if vec_count_match(&v2, 7) != 3 { return 5; }
  if vec_count_match(&v2, 99) != 0 { return 6; }
  return 0;
}
