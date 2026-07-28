// M35-Z14: recursion+generic+enum+match+Option+Result+contract+module+compound_assign+while
type Stack = { depth: Int; cur: Int; }
enum Opcode { Push(v: Int), Pop, Dup, Swap }
fn exec[T](op: Opcode, stk: Stack) -> Int
  requires: stk.depth >= 0
{
  match op {
    Push(v) => stk.depth + 1,
    Pop => { if stk.depth == 0 { return -1; } return stk.depth - 1; }
    Dup => { if stk.depth == 0 { return -1; } return stk.depth + 1; }
    Swap => { if stk.depth < 2 { return -1; } return stk.depth; }
  }
}
fn depth_from_ops(n: Int) -> Int {
  if n <= 0 { return 0; }
  var v = n + depth_from_ops(n - 1);
  return v;
}
module vm {
  pub fn step(o: Opcode, s: Stack) -> Int { return exec(o, s); }
  pub fn calc_depth(n: Int) -> Int { return depth_from_ops(n); }
}
use vm.step;
use vm.calc_depth;
fn main() -> Int {
  var s0 = Stack{ depth: 0; cur: 0; };
  var d1 = step(Opcode.Push(5), s0);
  if d1 < 0 { return 1; }
  var d2 = step(Opcode.Dup, Stack{ depth: d1; cur: 5; });
  if d2 < 0 { return 2; }
  var d3 = step(Opcode.Pop, Stack{ depth: d2; cur: 5; });
  if d3 < 0 { return 3; }
  var d = calc_depth(3);
  if d3 == 1 && d == 6 { return 0; }
  return 4;
}
