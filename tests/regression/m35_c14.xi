// M35-C14: infinite loop with break -- while true with multiple break conditions
fn collatz_steps(n: Int) -> Int {
  var x: Int = n;
  var steps: Int = 0;
  while true {
    if x == 1 { break; }
    if x % 2 == 0 { x = x / 2; }
    else { x = 3 * x + 1; }
    steps = steps + 1;
  }
  return steps;
}
fn main() -> Int {
  if collatz_steps(1) != 0 { return 1; }
  if collatz_steps(8) != 3 { return 2; }
  return 0;
}
