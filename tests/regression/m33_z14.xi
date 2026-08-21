// M33-Z14: Compound assignment on array element -- mutate via &mut reference
fn inc(cell: &mut Int, by: Int) { *cell += by; }
fn main() -> Int {
  var arr: Vec[Int] = [10, 20, 30];
  inc(&mut arr[0], 5);
  inc(&mut arr[1], 10);
  inc(&mut arr[2], -5);
  if arr[0] == 15 && arr[1] == 30 && arr[2] == 25 { return 0; }
  return 1;
}
