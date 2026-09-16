// fuzz seed: nested delimiters, generics, match, contracts.
module seed_parser

fn pick[T](a: T, b: T) -> T {
  match true {
    true => { return a; },
    false => { return b; },
    _ => { return a; },
  }
}

fn bounded(n: Int) -> Int
  requires: n >= 0
  ensures: result >= n
{
  var acc = n;
  while acc < 10 { acc = acc + (1 + 2 * 3); }
  return acc;
}

fn main() -> Int {
  return pick(bounded(1), 2);
}
