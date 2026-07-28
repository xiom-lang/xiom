// M36-X22: Mutual recursion — functions that call each other
fn is_even(n: Int) -> Bool {
  if n == 0 { return true; }
  return is_odd(n - 1);
}
fn is_odd(n: Int) -> Bool {
  if n == 0 { return false; }
  return is_even(n - 1);
}
fn ping(n: Int) -> Int {
  if n <= 0 { return 0; }
  return pong(n - 1) + 1;
}
fn pong(n: Int) -> Int {
  if n <= 0 { return 0; }
  return ping(n - 1) + 2;
}
fn ackermann(m: Int, n: Int) -> Int {
  if m == 0 { return n + 1; }
  if n == 0 { return ackermann(m - 1, 1); }
  return ackermann(m - 1, ackermann(m, n - 1));
}
fn main() -> Int {
  if !is_even(0) { return 1; }
  if !is_even(2) { return 2; }
  if is_even(3) { return 3; }
  if is_odd(0) { return 4; }
  if !is_odd(3) { return 5; }
  if is_odd(4) { return 6; }
  if ping(4) != 6 { return 7; }
  if ackermann(1, 2) != 4 { return 8; }
  return 0;
}
