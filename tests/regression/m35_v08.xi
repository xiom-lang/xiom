// M35-V08: Vec[Int] iteration over Vec with while loop
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.push(4);
  v.push(5);
  var sum = 0;
  var i = 0;
  while i < v.len() {
    sum = sum + v[i];
    i = i + 1;
  }
  if sum != 15 { return 1; }
  var product = 1;
  i = 0;
  while i < v.len() {
    product = product * v[i];
    i = i + 1;
  }
  if product != 120 { return 2; }
  return 0;
}
