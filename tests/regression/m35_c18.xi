// M35-C18: loop with flag -- boolean flag controls loop termination
fn first_prime_below(n: Int) -> Int {
  var i: Int = n;
  var found: Bool = false;
  var result: Int = 0;
  while i >= 2 && !found {
    var j: Int = 2;
    var is_prime: Bool = true;
    while j * j <= i && is_prime {
      if i % j == 0 { is_prime = false; }
      j = j + 1;
    }
    if is_prime { found = true; result = i; }
    i = i - 1;
  }
  return result;
}
fn main() -> Int {
  if first_prime_below(10) != 7 { return 1; }
  if first_prime_below(5) != 5 { return 2; }
  if first_prime_below(3) != 3 { return 3; }
  return 0;
}
