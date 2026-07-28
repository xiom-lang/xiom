// M36-E15: Numeric-suffixed names
fn f1(x: Int) -> Int { return x + 1; }
fn f2(x: Int) -> Int { return x + 2; }
fn main() -> Int {
  var v0 = 0; var v1 = f1(v0); var v2 = f2(v1);
  if v2 != 3 { return 1; }
  var data10 = 10; var data20 = 20;
  if data10 + data20 != 30 { return 2; }
  return 0;
}
