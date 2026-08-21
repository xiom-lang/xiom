module m37_contract_pass
// BUG 22 #5: contract checks must not break normal execution -- a
// SATISFIED requires/ensures still runs the body (regression guard for
// the xiom_panic clean-exit change; violations themselves print
// "contract violated: ..." to stderr and exit 1).

fn div_positive(a: Int, b: Int) -> Int
  requires: b > 0
  ensures: result == a / b
{
  return a / b;
}

fn main() -> Int {
  var r = div_positive(10, 2);
  if r != 5 { return 1; }
  var r2 = div_positive(100, 4);
  if r2 != 25 { return 2; }
  return 0;
}
