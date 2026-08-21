// M36-X07: Many function calls -- deep call chain stress test
fn f0(x: Int) -> Int { return x + 1; }
fn f1(x: Int) -> Int { return f0(x) + 1; }
fn f2(x: Int) -> Int { return f1(x) + 1; }
fn f3(x: Int) -> Int { return f2(x) + 1; }
fn f4(x: Int) -> Int { return f3(x) + 1; }
fn f5(x: Int) -> Int { return f4(x) + 1; }
fn f6(x: Int) -> Int { return f5(x) + 1; }
fn f7(x: Int) -> Int { return f6(x) + 1; }
fn f8(x: Int) -> Int { return f7(x) + 1; }
fn f9(x: Int) -> Int { return f8(x) + 1; }
fn f10(x: Int) -> Int { return f9(x) + 1; }
fn add_two(a: Int, b: Int) -> Int { return a + b; }
fn add_three(a: Int, b: Int, c: Int) -> Int { return a + b + c; }
fn compose(x: Int) -> Int { return f10(f5(f0(x))); }
fn chain_call(a: Int, b: Int, c: Int) -> Int {
  return add_two(add_two(a, b), add_two(c, 0));
}
fn main() -> Int {
  if f0(0) != 1 { return 1; }
  if f5(0) != 6 { return 2; }
  if f10(0) != 11 { return 3; }
  if compose(0) != 18 { return 4; }
  if add_three(1, 2, 3) != 6 { return 5; }
  if chain_call(1, 2, 3) != 6 { return 6; }
  return 0;
}
