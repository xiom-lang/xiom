// M36-C23: Every array pattern — Vec literal push, index, loop sum, param, return
fn array_param(arr: Vec[Int]) -> Int {
  if arr.len() == 0 { return -1; }
  return arr[0];
}
fn array_return() -> Vec[Int] {
  var v = Vec[Int].new();
  v.push(100);
  return v;
}
fn array_loop_sum(arr: Vec[Int]) -> Int {
  var s = 0;
  var i = 0;
  while i < arr.len() { s = s + arr[i]; i = i + 1; }
  return s;
}
fn main() -> Int {
  var a1 = Vec[Int].new();
  a1.push(42);
  if a1.len() != 1 { return 1; }
  if a1[0] != 42 { return 2; }
  var a3 = Vec[Int].new();
  a3.push(1); a3.push(2); a3.push(3);
  if a3.len() != 3 { return 3; }
  if a3[0] + a3[1] + a3[2] != 6 { return 4; }
  if array_param(a1) != 42 { return 5; }
  var v2 = Vec[Int].new();
  if array_param(v2) != -1 { return 6; }
  var rv = array_return();
  if rv.len() != 1 { return 7; }
  if rv[0] != 100 { return 8; }
  var vsum = Vec[Int].new();
  vsum.push(5); vsum.push(10); vsum.push(15);
  if array_loop_sum(vsum) != 30 { return 9; }
  var vempty = Vec[Int].new();
  if array_loop_sum(vempty) != 0 { return 10; }
  return 0;
}
