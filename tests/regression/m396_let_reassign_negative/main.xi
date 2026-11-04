// m396 lock (SPEC: "`let` cannot be reassigned. `var` can."):
// a `let` binding is immutable. The owner repro and the compound form
// (`+=` desugars to `x = x + ...`) must both be hard compile errors.
fn main() -> Int {
  let name = "xiom";
  name = "other";
  let n = 1;
  n += 1;
  return 0;
}
