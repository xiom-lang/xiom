// m150 companion (R-5): the T002 extern-confinement gate must still fire for
// a GENUINE extern "C" call in safe user code. The R-5 fix removes a name
// from the gate only when a fn DEFINITION with that name shadows the extern,
// so a plain extern call keeps requiring `unsafe`.
module m150_extern_gate;

extern "C" {
  fn m150_probe_abs(n: Int) -> Int;
}

fn main() -> Int {
  let x = m150_probe_abs(-3);
  return x;
}
