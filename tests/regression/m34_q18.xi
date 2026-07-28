// M34-Q18: Contract with closure — closures in contract-protected function chains
fn process_with_fn(f: fn(Int) -> Int, x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
{
  return f(x);
}
fn triple(x: Int) -> Int { return x * 3; }
fn add_five(x: Int) -> Int { return x + 5; }
fn chain_fns(x: Int) -> Int
  requires: x >= 0
  ensures: result >= x
{
  var a = process_with_fn(triple, x);
  var b = process_with_fn(add_five, a);
  return b;
}
fn main() -> Int {
  var r1 = chain_fns(5);
  var r2 = chain_fns(3);
  if r1 == 20 && r2 == 14 { return 0; }
  return 1;
}
