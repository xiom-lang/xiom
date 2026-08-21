// M35-V22: Vec sort-like check -- verify is_sorted predicate
fn is_sorted(v: &Vec[Int]) -> Bool {
  var i = 1;
  while i < v.len() {
    if v[i] < v[i - 1] { return false; }
    i = i + 1;
  }
  return true;
}

fn main() -> Int {
  var sorted = Vec[Int].new();
  sorted.push(1);
  sorted.push(2);
  sorted.push(3);
  sorted.push(4);
  if !is_sorted(&sorted) { return 1; }
  var unsorted = Vec[Int].new();
  unsorted.push(3);
  unsorted.push(1);
  unsorted.push(4);
  unsorted.push(2);
  if is_sorted(&unsorted) { return 2; }
  var single = Vec[Int].new();
  single.push(42);
  if !is_sorted(&single) { return 3; }
  var empty = Vec[Int].new();
  if !is_sorted(&empty) { return 4; }
  return 0;
}
