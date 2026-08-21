// M35-V24: Vec filter pattern -- keep only elements matching predicate
use xiom.collections;

fn filter_evens(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    if v[i] % 2 == 0 {
      result.push(v[i]);
    }
    i = i + 1;
  }
  return result;
}

fn filter_non_negative(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    if v[i] >= 0 {
      result.push(v[i]);
    }
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
  v.push(6);
  var evens = filter_evens(&v);
  if evens.len() != 3 { return 1; }
  if evens[0] != 2 { return 2; }
  if evens[1] != 4 { return 3; }
  if evens[2] != 6 { return 4; }

  var mixed = Vec[Int].new();
  mixed.push(-3);
  mixed.push(0);
  mixed.push(5);
  mixed.push(-1);
  mixed.push(8);
  var pos = filter_non_negative(&mixed);
  if pos.len() != 3 { return 5; }
  if pos[0] != 0 { return 6; }
  if pos[1] != 5 { return 7; }
  if pos[2] != 8 { return 8; }
  return 0;
}
