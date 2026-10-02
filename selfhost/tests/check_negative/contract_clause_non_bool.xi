// Phase 3 checker gate, negative case: contract clause must be Bool (strict
// mode is the default).
fn f(x: Int) -> Int
  requires: x
{
  return x;
}
