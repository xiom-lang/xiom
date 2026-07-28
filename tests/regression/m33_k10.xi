// M33-K10: Higher-order — function taking another function, verify with named fns
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn dbl(x: Int) -> Int { return x * 2; }
fn inc(x: Int) -> Int { return x + 1; }
fn main() -> Int {
  if apply(dbl, 10) != 20 { return 1; }
  if apply(inc, 41) != 42 { return 2; }
  return 0;
}
